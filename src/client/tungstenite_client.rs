use std::{ sync::Arc, time::Duration};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::{net::TcpStream, sync::RwLock, task::JoinHandle};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::{Message, client::IntoClientRequest}};
use super::{Bincoded, ClientMsg, ServerMsg};

pub struct TungsteniteClient {
    messages:Arc<RwLock<Vec<ServerMsg>>>,
    is_connected:Arc<RwLock<bool>>,
    reader:JoinHandle<()>,
    sink:Arc<RwLock<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>
}

fn spawn_reader(is_connected:Arc<RwLock<bool>>, messages:Arc<RwLock<Vec<ServerMsg>>>, web_socket:String, sink:Arc<RwLock<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>) -> JoinHandle<()> {
    let reader = tokio::spawn(async move {
        loop {
            while *is_connected.read().await == false {
                // not connected, try to connect
                if let Ok(req) = web_socket.clone().into_client_request() {
                    let conn = connect_async(req).await;
                    match conn {
                        Ok((ws_stream, _)) => {
                            // connected
                            {
                                // connected:
                                *is_connected.write().await = true;
                                let (s, mut stream) = ws_stream.split();
                                *sink.write().await = Some(s);

                                while let Some(msg) = stream.next().await {
                                    match msg {
                                        Ok(msg) => {
                                            if let Message::Binary(msg) = msg {
                                                if let Some(msg) = ServerMsg::from_bincode(&msg) {
                                                    messages.write().await.push(msg);
                                                } else {
                                                    break;
                                                }
                                            }
                                        },
                                        Err(_) => {
                                            break;
                                        },
                                    }
                                }

                                *is_connected.write().await = false;
                                *sink.write().await = None;
                            }
                        },
                        Err(_) => {
                            // failed, wait a bit and try again
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        },
                    }
                }
            }
        }
    });

    return reader;
}

impl TungsteniteClient {
    /// instantiate a Client using `Tokio-Tungstenite`
    ///
    /// `conn` is a websocket url, such as "ws://localhost:1234"
    pub fn new(websocket_url:&str) -> Option<Self> {
        let req = websocket_url.into_client_request();
        if let Ok(_) = req {
            let is_connected = Arc::new(RwLock::new(false));
            let messages = Arc::new(RwLock::new(Vec::with_capacity(128)));
            let sink = Arc::new(RwLock::new(None));
            let reader = spawn_reader(is_connected.clone(), messages.clone(), websocket_url.into(), sink.clone());

            return Some(Self {
                is_connected:is_connected.clone(),
                reader:reader,
                messages:messages,
                sink:sink
            });
        }

        None
    }


    pub fn close(&self) {
        self.reader.abort();
    }

    pub async fn is_connected(&self) -> bool {
        let c = self.is_connected.read().await;
        return *c;
    }

    pub async fn send(&mut self, msg:ClientMsg) -> bool {
        if self.is_connected().await {
            if let Some(sink) = &mut *self.sink.write().await {
                let res = sink.send(Message::Binary(msg.to_bincode())).await;
                match res {
                    Ok(_) => return true,
                    Err(_) => return false,
                }
            }
        }
        false
    }

    pub async fn pop_messages(&mut self) -> Vec<ServerMsg> {
        let cloned = self.messages.read().await.clone();
        self.messages.write().await.clear();
        return cloned;
    }
}

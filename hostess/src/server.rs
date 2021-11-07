use std::{net::SocketAddr, str::FromStr, sync::Arc};

use futures_util::{
    stream::{SplitSink, SplitStream},
     SinkExt, StreamExt,
};
use log::{error, info};
use tokio::{sync::RwLock, task::JoinHandle};
use uuid::Uuid;
use warp::{Error, Filter, ws::{Message, WebSocket}};

use crate::{Bincoded, ClientMsg, ServerMsg, lobby::Lobby, game_server::GameServerConstructor};

#[derive(Clone)]
pub struct ServerConfig {
    pub host_creation:bool,
    pub constructor:GameServerConstructor
}

pub struct Server {
    addr: String,
    pub lobby: Arc<RwLock<Lobby>>,
    pub config:ServerConfig
}

pub struct ConnectedClient {
    pub sink: ClientSink,
    pub stream: ClientStream,
    pub client_id:Uuid,
    pub client_name:String
}

pub struct ClientSink {
    sink:SplitSink<WebSocket, Message>,
}

impl ClientSink {
    pub async fn send(&mut self, msg:ServerMsg) -> Result<(), Error> {
        self.sink.send(Message::binary(msg.to_bincode())).await
    }
}

impl From<SplitSink<WebSocket, Message>> for ClientSink {
    fn from(sink: SplitSink<WebSocket, Message>) -> Self {
        Self {
            sink
        }
    }
}
pub struct ClientStream {
    stream: SplitStream<WebSocket>
}

impl ClientStream {
    pub async fn next<'a, T : Bincoded>(&'a mut self) -> Option<Result<T, Box<dyn std::error::Error + Send>>> {
        match self.stream.next().await {
            Some(msg) => {
                match msg {
                    Ok(msg) => {
                        let bytes = msg.as_bytes();//.as_bytes();
                        match T::from_bincode(bytes) {
                            Some(msg) => {
                                return Some(Ok(msg));
                            },
                            None => {
                                None 
                            },
                        }

                    },
                    Err(err) => {
                        return Some(Err(Box::new(err)));
                    },
                }
            },
            None => {
                return None;
            },
        }
    }
}

impl From<SplitStream<WebSocket>> for ClientStream {
    fn from(stream: SplitStream<WebSocket>) -> Self {
        Self {
            stream
        }
    }
}

impl Server {
    pub fn new(addr: &str, constructor:GameServerConstructor) -> Self {
        Self {
            addr: addr.into(),
            lobby: Arc::new(RwLock::new(Lobby::new())),
            config:ServerConfig { host_creation: false, constructor:constructor }
        }
    }

    async fn client_joined_lobby(
        mut client:ConnectedClient,
        lobby: Arc<RwLock<Lobby>>,
        config:ServerConfig
    ) {
        info!("Client {:?} entered lobby", client.client_id);

        // send list of hosts to client
        let _ = client.sink.send(ServerMsg::Hosts {
            hosts:lobby.read().await.hosts()
        }).await;

        while let Some(msg) = client.stream.stream.next().await {
            match msg {
                Ok(msg) => {
                    let bytes = msg.as_bytes();
                    if bytes.len() > 0 {
                        match bincode::deserialize::<ClientMsg>(bytes) {
                            Ok(msg) => {
                                match msg {
                                    ClientMsg::CreateHost {} => {
                                        if config.host_creation {
                                            // create new host
                                            let mut lobby = lobby.write().await;
                                            let host_id = lobby.new_host(client.client_id, config.constructor.clone());

                                            // and tell this to the  client
                                            let _ = client.sink.send(ServerMsg::HostCreated {
                                                host_id:host_id
                                            }).await;
                                        }
                                    },
                                    ClientMsg::RefreshHosts => {
                                        let _ = client.sink.send(ServerMsg::Hosts {
                                            hosts:lobby.read().await.hosts()
                                        }).await;
                                    },
                                    ClientMsg::JoinHost { host_id } => {
                                        let lobby = lobby.read().await;
                                        if let Some(host) = lobby.get_host(host_id) {
                                            if let Some(c) = host.join(client).await {
                                                client = c;
                                            } else {
                                                break;
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            },
                            Err(err) => {
                                error!("{:?}", err);
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("{:?}", err);
                    break;
                }
            }
        }
    }


    async fn client_connected(ws: WebSocket, lobby: Arc<RwLock<Lobby>>, config:ServerConfig) {
        let (tx, rx) = ws.split();
        let mut tx:ClientSink = tx.into();
        let mut stream:ClientStream = rx.into();

        let mut id = None;
        let mut name = "".into();

        // wait for Hello message to get client id
        while let Some(msg) = stream.stream.next().await {
            match msg {
                Ok(msg) => {
                    let bytes = msg.as_bytes();
                    if bytes.len() > 0 {
                        match bincode::deserialize::<ClientMsg>(bytes) {
                            Ok(msg) => match msg {
                                ClientMsg::Hello { client_id, client_name } => {
                                    id = Some(client_id);
                                    name = client_name;
                                    break;
                                }
                                _ => {}
                            },
                            Err(err) => {
                                error!("{:?}", err);
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("{:?}", err);
                    break;
                }
            }
        }

        if let Some(client_id) = id {
            // Hello received, send Welcome message
            // and proceed to lobby if successfull
            let msg = ServerMsg::LobbyJoined {};
            match tx.send(msg).await {
                Ok(_) => {
                    Self::client_joined_lobby(ConnectedClient{sink: tx, stream, client_id, client_name: name}, lobby, config).await
                },
                Err(_) => error!("Client {} failed to join", client_id),
            }
        }

        // no more, disconnect the client

        match id {
            Some(id) => info!("Client {} disconnected", id),
            None => info!("Unknown Client disconnected"),
        }
    }

    pub fn spawn(self) -> JoinHandle<()> {
        return tokio::spawn(async move {
            let addr = SocketAddr::from_str(&self.addr).expect("Could not parse address");

            let public_route = warp::fs::dir("./public");
            let lobby = self.lobby.clone();
            let config = self.config.clone();
            let ws_route = warp::ws().map(move |ws: warp::ws::Ws| {
                let lobby = lobby.clone();
                let config = config.clone();
                ws.on_upgrade(move |ws| Self::client_connected(ws, lobby, config))
            });

            let routes = warp::get().and(ws_route).or(public_route);

            warp::serve(routes).run(addr).await;
        });
    }
}

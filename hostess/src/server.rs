use std::{marker::PhantomData, net::SocketAddr, str::FromStr, sync::Arc};

use futures_util::{
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt,
};
use log::{error, info};
use tokio::{sync::RwLock, task::JoinHandle};
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

use crate::{game::Game, lobby::Lobby, ClientMsg, ServerMsg};

pub struct Server<T> {
    addr: String,
    phantom: PhantomData<T>,
    lobby: Arc<RwLock<Lobby>>,
}

struct ConnectedClient {
    pub tx: SplitSink<WebSocket, Message>,
    pub rx: SplitStream<WebSocket>,
    pub client_id:Uuid
}

impl<T: Game + Send + 'static> Server<T> {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.into(),
            phantom: PhantomData::default(),
            lobby: Arc::new(RwLock::new(Lobby::new())),
        }
    }

    async fn client_joined_lobby(
        mut client:ConnectedClient,
        lobby: Arc<RwLock<Lobby>>,
    ) {
        info!("Client {:?} entered lobby", client.client_id);

        while let Some(msg) = client.rx.next().await {
            match msg {
                Ok(msg) => {
                    let bytes = msg.as_bytes();
                    if bytes.len() > 0 {
                        match bincode::deserialize::<ClientMsg>(bytes) {
                            Ok(msg) => {
                                match msg {
                                    ClientMsg::CreateHost {} => {
                                        // create new host
                                        let mut lobby = lobby.write().await;
                                        let host_id = lobby.new_host(client.client_id);
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


    async fn client_connected(ws: WebSocket, lobby: Arc<RwLock<Lobby>>) {
        let (mut tx, mut rx) = ws.split();

        let mut id = None;

        // wait for Hello message to get client id
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(msg) => {
                    let bytes = msg.as_bytes();
                    if bytes.len() > 0 {
                        match bincode::deserialize::<ClientMsg>(bytes) {
                            Ok(msg) => match msg {
                                ClientMsg::Hello { client_id } => {
                                    id = Some(client_id);
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
            let msg = ServerMsg::Welcome {};
            match tx.send(Message::binary(msg.to_bincode())).await {
                Ok(_) => {
                    Self::client_joined_lobby(ConnectedClient{tx, rx, client_id}, lobby).await
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
            let ws_route = warp::ws().map(move |ws: warp::ws::Ws| {
                let lobby = lobby.clone();
                ws.on_upgrade(move |ws| Self::client_connected(ws, lobby))
            });

            let routes = warp::get().and(ws_route).or(public_route);

            warp::serve(routes).run(addr).await;
        });
    }
}

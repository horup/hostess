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

#[derive(Clone, Copy, Default)]
pub struct ServerConfig {
    pub host_creation:bool
}

pub struct Server<T:Game> {
    addr: String,
    phantom: PhantomData<T>,
    pub lobby: Arc<RwLock<Lobby<T>>>,
    pub config:ServerConfig
}

pub struct ConnectedClient {
    pub tx: SplitSink<WebSocket, Message>,
    pub rx: SplitStream<WebSocket>,
    pub client_id:Uuid
}

impl ConnectedClient {
    pub async fn send(&mut self, server_msg:ServerMsg) {
        let _ = self.tx.send(Message::binary(server_msg.to_bincode())).await;
    }
}

impl<T: Game> Server<T> {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.into(),
            phantom: PhantomData::default(),
            lobby: Arc::new(RwLock::new(Lobby::new())),
            config:ServerConfig::default()
        }
    }

    async fn client_joined_lobby(
        mut client:ConnectedClient,
        lobby: Arc<RwLock<Lobby<T>>>,
        config:ServerConfig
    ) {
        info!("Client {:?} entered lobby", client.client_id);

        // send list of hosts to client
        client.send(ServerMsg::Hosts {
            hosts:lobby.read().await.hosts()
        }).await;

        while let Some(msg) = client.rx.next().await {
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
                                            let host_id = lobby.new_host(client.client_id);

                                            // and tell this to the  client
                                            client.send(ServerMsg::HostCreated {
                                                host_id:host_id
                                            }).await;
                                        }
                                    },
                                    ClientMsg::RefreshHosts => {
                                        client.send(ServerMsg::Hosts {
                                            hosts:lobby.read().await.hosts()
                                        }).await;
                                    },
                                    ClientMsg::JoinHost { host_id } => {
                                        let mut lobby = lobby.write().await;
                                        if let Some(host) = lobby.get_host_mut(host_id) {
                                            host.join(&mut client).await;
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


    async fn client_connected(ws: WebSocket, lobby: Arc<RwLock<Lobby<T>>>, config:ServerConfig) {
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
            let msg = ServerMsg::LobbyJoined {};
            match tx.send(Message::binary(msg.to_bincode())).await {
                Ok(_) => {
                    Self::client_joined_lobby(ConnectedClient{tx, rx, client_id}, lobby, config).await
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

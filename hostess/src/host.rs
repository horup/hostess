use std::{collections::HashMap, sync::{Arc}, time::Duration};

use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::{sync::{RwLock, mpsc::channel, mpsc::Sender, mpsc::Receiver}, time::{interval}};
use uuid::Uuid;
use log::{info};
use warp::ws::{Message, WebSocket};

use crate::{ClientMsg, ConnectedClient, Context, Game, GameMsg, HostInfo, HostMsg, ServerMsg};

enum Msg {
    HostMsg(HostMsg),
    ClientTransfer {
        client_id:Uuid,
        tx:SplitSink<WebSocket, Message>,
        return_tx:tokio::sync::oneshot::Sender<SplitSink<WebSocket, Message>>
    }
}
pub struct Host {
    pub info:HostInfo,
    //pub messages_to_game:Arc<RwLock<Vec<HostMsg>>>
    //pub HashMap<Uuid,  
    sender:Sender<Msg>,
}

impl Host {
    pub fn new<T:Game>(info:HostInfo) -> Self {
        let buffer_len = 1024;
        let (sender, mut receiver) = channel::<Msg>(buffer_len);
        let host = Self {
            info,
            sender,
        };

        tokio::spawn(async move {
            let mut g = T::new();
            let period = Duration::from_millis(1000 / g.tick_rate());
            let mut timer = interval(period);
            let mut run = true;

            let mut context = Context {
                game_messages:Vec::new(),
                host_messages:Vec::with_capacity(buffer_len)
            };

            let mut clients:HashMap<Uuid, (SplitSink<WebSocket, Message>, tokio::sync::oneshot::Sender<SplitSink<WebSocket, Message>>)> = HashMap::new();

            loop {
                //receiver.poll_recv(cx)
                loop {
                    match receiver.try_recv() {
                        Ok(msg) => {
                            match msg {
                                Msg::HostMsg(msg) => {
                                    info!("{:?}", msg);
                                    match &msg {
                                        HostMsg::ClientLeft { client_id } => {
                                            if let Some((tx, transfer)) = clients.remove(client_id) {
                                                transfer.send(tx);
                                            }
                                        },
                                        HostMsg::CustomMsg { client_id, msg } => {

                                        },
                                        _=>{}
                                    }

                                    context.host_messages.push(msg);
                                },
                                Msg::ClientTransfer { 
                                    client_id, 
                                    tx, 
                                    return_tx 
                                } => {
                                    clients.insert(client_id, (tx, return_tx));
                                },
                            }
                        },
                        Err(_) => {
                            break;
                        },
                    }
                }

                //messages_to_game.read().await;
                g.update(&mut context);

                for msg in &context.game_messages {

                }
                timer.tick().await;
            }
        });

        host
    }

    pub async fn join(&self, mut client:ConnectedClient) -> ConnectedClient {
        let mut tx = client.tx;
        let mut rx = client.rx;
        
       /* let _ = tx.send(Message::binary(ServerMsg::HostJoined {
            host:self.info.clone()
        }.to_bincode())).await;



        info!("Client {} joined Host {}", client.client_id, self.info.id);

        let host_sender = self.sender.clone();
        let _ = host_sender.send(Msg::HostMsg(HostMsg::ClientJoined {
            client_id:client.client_id
        })).await;*/

        let (return_tx, return_rx) = tokio::sync::oneshot::channel::<SplitSink<WebSocket, Message>>();
        let host_sender = self.sender.clone();
        let _ = host_sender.send(Msg::ClientTransfer {
            client_id: client.client_id,
            tx,
            return_tx,
        }).await;

        // while part of host
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(msg) => {
                    let bytes = msg.as_bytes();
                    match ClientMsg::from_bincode(bytes) {
                        Some(msg) => {
                            match msg {
                                ClientMsg::LeaveHost {} => {
                                    // exit while and leave host
                                    break;
                                },
                                ClientMsg::CustomMsg {
                                    msg
                                } => {
                                    let _ = host_sender.send(Msg::HostMsg(HostMsg::CustomMsg {
                                        client_id:client.client_id,
                                        msg
                                    })).await;
                                }
                                _ => {}
                            }
                        },
                        None => {
                            break
                        },
                    }
                },
                Err(_) => break,
            }
        }

        let _ = host_sender.send(Msg::HostMsg(HostMsg::ClientLeft {
            client_id:client.client_id
        })).await;

        let tx = return_rx.await.expect("needs to be handled gracefully...");

        info!("Client {} left Host {}", client.client_id, self.info.id);

        ConnectedClient {
            tx,
            rx,
            client_id:client.client_id
        }
    }
}
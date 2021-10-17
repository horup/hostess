use std::{collections::{HashMap, VecDeque}, sync::{Arc}, time::Duration};

use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::{sync::{RwLock, mpsc::channel, mpsc::Sender, mpsc::Receiver}, time::{interval}};
use uuid::Uuid;
use log::{info};
use warp::ws::{Message, WebSocket};

use crate::{Bincode, ClientMsg, ConnectedClient, Context, Game, GameMsg, HostInfo, HostMsg, ServerMsg};

enum Msg {
    HostMsg(HostMsg),
    ClientTransfer {
        client_id:Uuid,
        tx:SplitSink<WebSocket, Message>,
        return_tx:tokio::sync::oneshot::Sender<SplitSink<WebSocket, Message>>
    }
}
#[derive(Clone)]
pub struct Host {
    pub info:HostInfo,
    sender:Sender<Msg>,
}

impl Host {
    pub fn new<T:Game>(info:HostInfo) -> Self {
        let buffer_len = 1024;
        let (sender, mut receiver) = channel::<Msg>(buffer_len);
        let host = Self {
            info:info.clone(),
            sender,
        };

        tokio::spawn(async move {
            let mut g = T::new();
            let period = Duration::from_millis(1000 / g.tick_rate());
            let mut timer = interval(period);
            let mut run = true;

            let mut context = Context {
                game_messages:VecDeque::new(),
                host_messages:Vec::with_capacity(buffer_len)
            };

            let mut clients:HashMap<Uuid, (SplitSink<WebSocket, Message>, tokio::sync::oneshot::Sender<SplitSink<WebSocket, Message>>)> = HashMap::new();

            loop {
                loop {
                    match receiver.try_recv() {
                        Ok(msg) => {
                            match msg {
                                Msg::HostMsg(msg) => {
                                    info!("{:?}", msg);
                                    match &msg {
                                        HostMsg::ClientLeft { client_id } => {
                                            if let Some((tx, transfer)) = clients.remove(client_id) {
                                                let _ = transfer.send(tx);
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
                                    mut tx, 
                                    return_tx 
                                } => {
                                    context.host_messages.push(HostMsg::ClientJoined {
                                        client_id:client_id
                                    });
                                    let _ = tx.send(Message::binary(ServerMsg::HostJoined {
                                        host:info.clone()
                                    }.to_bincode())).await;

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

                // TODO: can be fuffered maybe?
                for msg in context.game_messages.drain(..) {
                    match msg {
                        GameMsg::CustomToAll { msg } => {
                            for (sink, _) in &mut clients.values_mut() {
                                let _ = sink.send(Message::binary(ServerMsg::Custom{
                                    msg:msg.clone()
                                }.to_bincode())).await;
                            }
                        },
                        GameMsg::CustomTo { client_id, msg } => {
                            todo!();
                        },
                    }
                }
                
                //while let Some(msg) in context.game_messages.remove(0)
                timer.tick().await;
            }
        });

        host
    }

    pub async fn join(&self, client:ConnectedClient) -> Option<ConnectedClient> {
        info!("Client {} joined Host {}", client.client_id, self.info.id);
        let tx = client.tx;
        let mut rx = client.rx;

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
        
        info!("Client {} left Host {}", client.client_id, self.info.id);
        if let Ok(tx) = return_rx.await {
            return Some(ConnectedClient {
                tx,
                rx,
                client_id:client.client_id
            });
        };

        None
    }
}
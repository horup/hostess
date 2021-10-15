use std::{collections::HashMap, sync::{Arc}, time::Duration};

use futures_util::StreamExt;
use tokio::{sync::{RwLock, mpsc::channel, mpsc::Sender, mpsc::Receiver}, time::{interval}};
use uuid::Uuid;
use log::{info};

use crate::{ClientMsg, ConnectedClient, Context, Game, GameMsg, HostInfo, HostMsg, ServerMsg};

pub struct Host {
    pub info:HostInfo,
    //pub messages_to_game:Arc<RwLock<Vec<HostMsg>>>
    //pub HashMap<Uuid,  
    pub sender:Sender<HostMsg>
}

impl Host {
    pub fn new<T:Game>(info:HostInfo) -> Self {
        let buffer_len = 1024;
        let (sender, mut receiver) = channel::<HostMsg>(buffer_len);
        let host = Self {
            info,
            sender
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

            loop {
                //receiver.poll_recv(cx)
                loop {
                    match receiver.try_recv() {
                        Ok(msg) => {
                            info!("{:?}", msg);
                            context.host_messages.push(msg);
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

    pub async fn join(&self, mut client:&mut ConnectedClient) {
        client.send(ServerMsg::HostJoined {
            host:self.info.clone()
        }).await;

        info!("Client {} joined Host {}", client.client_id, self.info.id);

        let host_sender = self.sender.clone();
        let _ = host_sender.send(HostMsg::ClientJoined {
            client_id:client.client_id
        }).await;

        // while part of host
        while let Some(msg) = client.rx.next().await {
            match msg {
                Ok(msg) => {
                    let bytes = msg.as_bytes();
                    match ClientMsg::from_bincode(bytes) {
                        Some(msg) => {
                            match msg {
                                ClientMsg::LeaveHost {} => {
                                    break;
                                },
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

        let _ = host_sender.send(HostMsg::ClientLeft {
            client_id:client.client_id
        }).await;

        info!("Client {} left Host {}", client.client_id, self.info.id);
    }
}
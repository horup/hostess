use std::{collections::{HashMap, VecDeque}, sync::Arc, time::{Duration, Instant}};

use futures_util::{FutureExt, pin_mut};
use tokio::{sync::{RwLock, mpsc::Sender, mpsc::channel}, time::{MissedTickBehavior, interval}};
use uuid::Uuid;
use log::{info};
use tokio::select;
use crate::shared::{HostInfo};

use crate::{client::{ClientMsg, ServerMsg}, server::{Constructor, Ctx, GameServerMsg, HostMsg}, master::{ClientSink, Client}};

enum Msg {
    HostMsg(HostMsg),
    ClientTransfer {
        client_id:Uuid,
        client_name:String,
        sink:ClientSink,
        return_sink:tokio::sync::oneshot::Sender<ClientSink>
    },
    Ping {
        client_id:Uuid,
        tick:f64
    }
}
#[derive(Clone)]
pub struct Host {
    pub info:Arc<RwLock<HostInfo>>,
    sender:Sender<Msg>,
}

impl Host {
    pub fn new(host_info:Arc<RwLock<HostInfo>>, constructor:Constructor) -> Self {
        let buffer_len = 1024;
        let (sender, mut receiver) = channel::<Msg>(buffer_len);

        let host = Self {
            info:host_info.clone(),
            sender,
        };

        tokio::spawn(async move {
            let mut g = constructor.construct();
            let config = g.init();
            {
                let mut host_info = host_info.write().await;
                host_info.current_players = 0;
                host_info.max_players = config.max_players;
            }

            let period = Duration::from_millis(1000 / config.tick_rate);
            let mut timer = interval(period);
            timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let mut context = Ctx {
                game_messages:VecDeque::new(),
                host_messages:VecDeque::with_capacity(buffer_len),
                delta:timer.period().as_secs_f64(),
                time:0.0
            };

            let mut clients:HashMap<Uuid, (ClientSink, tokio::sync::oneshot::Sender<ClientSink>)> = HashMap::new();

            let mut last_tick = Instant::now();
            loop {
                let timer = timer.tick().fuse();//.await;
                let recv = receiver.recv().fuse();
                pin_mut!(timer, recv);
                select! {
                    _ = timer => {
                        let now = Instant::now();
                        let diff = now - last_tick;
                        context.delta = diff.as_secs_f64();
                        context.time += context.delta;
                        g.tick(&mut context);
                        for msg in context.game_messages.drain(..) {
                            match msg {
                                GameServerMsg::CustomToAll { msg } => {
                                    for (sink, _) in &mut clients.values_mut() {
                                        let _ = sink.send(ServerMsg::Custom{
                                            msg:msg.clone()
                                        }).await;
                                    }
                                },
                                GameServerMsg::CustomTo { client_id, msg } => {
                                    if let Some((sink, _)) = clients.get_mut(&client_id) {
                                        let _ = sink.send(ServerMsg::Custom{
                                            msg:msg.clone()
                                        }).await;
                                    }
                                },
                            }
                        }

                        context.host_messages.clear();

                        last_tick = Instant::now();
                    },
                    msg = recv => {
                        match msg {
                            Some(msg) => {
                                match msg {
                                    Msg::HostMsg(msg) => {
                                        match &msg {
                                            HostMsg::ClientLeft { client_id } => {
                                                if let Some((tx, transfer)) = clients.remove(client_id) {
                                                    let mut host_info = host_info.write().await;
                                                    host_info.current_players -= 1;
                                                    let _ = transfer.send(tx);
                                                }
                                            },
                                            _=>{}
                                        }
    
                                        context.host_messages.push_back(msg);
                                    },
                                    Msg::ClientTransfer { 
                                        client_id, 
                                        client_name,
                                        sink: mut tx, 
                                        return_sink: return_tx 
                                    } => {
                                        let mut host_info = host_info.write().await;
                                        if host_info.current_players >= host_info.max_players {
                                            // if max players reach, reject.
                                            let _ = tx.send(ServerMsg::JoinRejected {
                                                host:host_info.clone()
                                            }).await;

                                            let _ = return_tx.send(tx);
                                        } else {
                                            // else accept the join
                                            context.host_messages.push_back(HostMsg::ClientJoined {
                                                client_id:client_id,
                                                client_name:client_name
                                            });
                                            host_info.current_players += 1;
                                            let _ = tx.send(ServerMsg::HostJoined {
                                                host:host_info.clone()
                                            }).await;
        
                                            clients.insert(client_id, (tx, return_tx));
                                        }
                                    },
                                    Msg::Ping {
                                        client_id,
                                        tick
                                    } => {
                                        if let Some((tx, _)) = clients.get_mut(&client_id) {
                                            let server_bytes_sec = tx.bytes_per_second.per_second();
                                            let _ = tx.send(ServerMsg::Pong {
                                                tick:tick,
                                                server_bytes_sec:server_bytes_sec,
                                                client_bytes_sec:server_bytes_sec
                                            }).await;
                                        }
                                    }
                                }
                            },
                            None => {}
                        }
                    }
                };
            }
        });

        return host;
    }

    pub async fn join(&self, client:Client) -> Option<Client> {
        info!("Client {} with name '{}' joined Host {}", client.client_id, client.client_name, self.info.read().await.id);
        let tx = client.sink;
        let mut rx = client.stream;

        let (return_tx, return_rx) = tokio::sync::oneshot::channel::<ClientSink>();
        let host_sender = self.sender.clone();
        let _ = host_sender.send(Msg::ClientTransfer {
            client_id: client.client_id,
            client_name:client.client_name.clone(),
            sink: tx,
            return_sink: return_tx,
        }).await;

        while let Some(msg) = rx.next::<ClientMsg>().await {
            match msg {
                Ok(msg) => {
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
                        },
                        ClientMsg::Ping {
                            tick
                        } => {
                            let _ = host_sender.send(Msg::Ping {
                                client_id:client.client_id,
                                tick:tick
                            }).await;
                        }
                        _ => {}
                    }
                },
                Err(_) => {
                    break;
                },
            }
        }

        let _ = host_sender.send(Msg::HostMsg(HostMsg::ClientLeft {
            client_id:client.client_id
        })).await;
        
        info!("Client {} left Host {}", client.client_id, self.info.read().await.id);
        if let Ok(tx) = return_rx.await {
            return Some(Client {
                sink: tx,
                stream: rx,
                client_id:client.client_id,
                client_name:client.client_name
            });
        };

        None
    }
}
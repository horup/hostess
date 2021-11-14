use std::{collections::{HashMap, VecDeque}, time::{Duration, Instant}};

use futures_util::{FutureExt, pin_mut};
use tokio::{sync::{mpsc::channel, mpsc::Sender}, time::{MissedTickBehavior, interval}};
use uuid::Uuid;
use log::{info};
use tokio::select;

use crate::{ClientMsg, HostInfo, ServerMsg, server::{ClientSink, ConnectedClient}, game_server::{GameServerConstructor, Context, GameServerMsg, HostMsg}};

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
    pub info:HostInfo,
    sender:Sender<Msg>,
}

impl Host {
    pub fn new(info:HostInfo, constructor:GameServerConstructor) -> Self {
        let buffer_len = 1024;
        let (sender, mut receiver) = channel::<Msg>(buffer_len);
        let host = Self {
            info:info.clone(),
            sender,
        };

        tokio::spawn(async move {
            let mut g = constructor.construct();
            let period = Duration::from_millis(1000 / g.tick_rate());
            let mut timer = interval(period);
            timer.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut context = Context {
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
                        context = g.tick(context);
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
                                        context.host_messages.push_back(HostMsg::ClientJoined {
                                            client_id:client_id,
                                            client_name:client_name
                                        });
                                        let _ = tx.send(ServerMsg::HostJoined {
                                            host:info.clone()
                                        }).await;
    
                                        clients.insert(client_id, (tx, return_tx));
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

        host
    }

    pub async fn join(&self, client:ConnectedClient) -> Option<ConnectedClient> {
        info!("Client {} with name '{}' joined Host {}", client.client_id, client.client_name, self.info.id);
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
        
        info!("Client {} left Host {}", client.client_id, self.info.id);
        if let Ok(tx) = return_rx.await {
            return Some(ConnectedClient {
                sink: tx,
                stream: rx,
                client_id:client.client_id,
                client_name:client.client_name
            });
        };

        None
    }
}
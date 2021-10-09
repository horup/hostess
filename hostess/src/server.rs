use std::{marker::PhantomData, net::SocketAddr, str::FromStr};

use futures_util::{
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt,
};
use log::info;
use tokio::task::JoinHandle;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

use crate::game::Game;

pub struct Server<T> {
    addr: String,
    phantom: PhantomData<T>,
}

impl<T: Game + Send + 'static> Server<T> {
    pub fn new(addr: &str) -> Self {
        Self {
            addr:addr.into(),
            phantom: PhantomData::default(),
        }
    }

    /* async fn client_joined(mut tx: SplitSink<WebSocket, Message>, mut rx:SplitStream<WebSocket>, bus:Bus, client_id:Uuid) {
        info!("Client {:?} joined", client_id);

        while let Some(msg) = rx.next().await {

        }
    }*/

    async fn client_connected(ws: WebSocket) {
        let (mut tx, mut rx) = ws.split();

        //let mut id = None;

        // wait for Hello message to get client id
        /*     while let Some(msg) = rx.next().await {
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
        }*/

        /*  if let Some(id) = id {
            let msg = ServerMsg::Welcome {

            };


            match tx.send(Message::binary(msg.to_bincode())).await {
                Ok(_) => Self::client_joined(tx, rx, bus, id).await,
                Err(_) => error!("Client {} failed to join", id),
            }
        }


        match id {
            Some(id) => info!("Client {} disconnected", id),
            None => info!("Unknown Client disconnected")
        }*/
    }

    pub fn spawn(self) -> JoinHandle<()> {
        return tokio::spawn(async move {
            let addr = SocketAddr::from_str(&self.addr).expect("Could not parse address");

            let public_route = warp::fs::dir("./public");

            let ws_route = warp::ws()
                .map(move |ws: warp::ws::Ws| ws.on_upgrade(move |ws| Self::client_connected(ws)));

            let routes = warp::get().and(ws_route).or(public_route);

            warp::serve(routes).run(addr).await;
        });
    }
}

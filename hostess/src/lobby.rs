use futures_util::{StreamExt, stream::{SplitSink, SplitStream}};
use log::info;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub struct Lobby {

}

impl Lobby {
    pub fn new() -> Self {
        Lobby {

        }
    }

    pub async fn client_entered(mut tx: SplitSink<WebSocket, Message>, mut rx: SplitStream<WebSocket>, client_id:Uuid) {
        info!("Client {:?} entered lobby", client_id);

        // process messages
        while let Some(msg) = rx.next().await {}

        // all done
    }
}
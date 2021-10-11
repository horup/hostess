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

    pub fn new_host(&mut self, creator:Uuid) -> Uuid {
        let host_id = Uuid::new_v4();
        info!("Host {:?} created by client {}", host_id, creator);
        return host_id;
    }
}
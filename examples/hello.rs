use std::time::Duration;

use hostess::{client::{ClientMsg, ServerMsg, tungstenite_client::TungsteniteClient}, master::Master, server::*, uuid::*};
use tokio::task::JoinHandle;
#[derive(Default)]
struct HelloServer {
    pub players:u32
}

impl Server for HelloServer {
    fn init(&mut self) -> Config {
        Config {
            tick_rate: 5,
            max_players: 8,
        }
    }

    fn tick(&mut self, ctx:&mut Ctx) {
        while let Some(msg) = ctx.pop_msg() {
            match msg {
                InMsg::ClientJoined { client_id: _, client_name: _ } => self.players += 1,
                InMsg::ClientLeft { client_id: _ } => self.players -= 1,
                InMsg::CustomMsg { client_id: _, msg:_ } => {
                },
            }
        }
        println!("players:{}", self.players);
    }
}

const ADDR:&str = "127.0.0.1:1234";
pub fn spawn_client() -> JoinHandle<()> {
    tokio::spawn(async {
        let addr = format!("ws://{}", ADDR);
        let mut client = TungsteniteClient::new(&addr).unwrap();

        client.connect().await;

        // send hello
        let res = client.send(ClientMsg::Hello {
            client_id: Uuid::new_v4(),
            client_name: "Test Client".into(),
        }).await;

        loop {
            // consume messages from server
            for msg in client.messages().await.unwrap().iter() {
                match msg {
                    ServerMsg::Instances {instances: hosts} => {
                        // join host
                        client.send(ClientMsg::JoinInstance {
                            instance_id: hosts.first().unwrap().id,
                        }).await;
                    },
                    ServerMsg::JoinRejected {
                        instance: host
                    } => {
                        println!("failed to join host {:?}", host);
                    }
                    _ => {}
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
}

pub fn spawn_master() -> JoinHandle<()> {
    tokio::spawn(async {
        let mut master = Master::new(ADDR, Constructor::new::<HelloServer>());
        master.new_server(Uuid::default()).await;
        let _ = master.start().await;
    })
}

#[tokio::main]
pub async fn main() {
    let master = spawn_master();

    for _ in 0..10 {
        spawn_client();
    }

    let _ = master.await;
}
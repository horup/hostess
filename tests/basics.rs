use std::{process::exit};
use futures_util::{ SinkExt, Stream, StreamExt};
use hostess::{bincoded::Bincoded, client::{ClientMsg, ServerMsg}, server::{Config, Server, Constructor, InMsg, OutMsg, Ctx}, master::Master};
use tokio::{time::Duration};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
   
};
use uuid::Uuid;

#[derive(Default)]
pub struct TestGame {
    pub client_id:Option<Uuid>
}

impl Server for TestGame {
    fn tick(&mut self, context: &mut Ctx) {
        let messages = context.pop_all();
        for msg in messages.iter() {
            match msg {
                InMsg::ClientJoined { client_id, client_name } => {
                    assert_eq!(client_name, "Tester");
                    self.client_id = Some(client_id.clone());
                },
                InMsg::ClientLeft { client_id } => {
                    assert_eq!(self.client_id.unwrap(), *client_id);
                    self.client_id = None;
                },
                InMsg::CustomMsg { client_id, msg } => {
                    assert_eq!(self.client_id.unwrap(), *client_id);
                    context.push_msg(OutMsg::CustomTo {
                        client_id: *client_id,
                        msg: msg.clone(),
                    });
                },
            }
        }
    }

    fn init(&mut self) -> hostess::server::Config {
        Config {
            tick_rate:20,
            max_players:1
        }
    }
}

impl TestGame {
    pub fn constructor() -> Constructor {
        Constructor::new::<Self>()
    }
}

async fn send<T: SinkExt<Message> + Unpin>(t: &mut T, msg: ClientMsg) {
    let _ = t.send(Message::binary(msg.to_bincode())).await;
}

async fn recv<T: Unpin + Stream<Item = Result<Message, U>>, U : std::fmt::Debug>(t: &mut T) -> ServerMsg {
    let res = t.next().await.unwrap().unwrap();
    match res {
        Message::Binary(b) => {
            return Bincoded::from_bincode(&b).unwrap();
        }
        _ => panic!(),
    }
}

const LISTEN: &str = "127.0.0.1:8080";
#[tokio::test]
pub async fn basics() {
    // setup watchdog to ensure test exists
    tokio::spawn( async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        exit(1);
    });

    // create a manager with some game_servers
    let _ = tokio::spawn(async {
        let mut master = Master::new(LISTEN, TestGame::constructor());

        for _ in 0..10 {
            master.new_instance(Uuid::default())
                .await;
        }

        let _ = master.start().await;
    });

    tokio::time::sleep(Duration::from_millis(1000)).await;

    // connect to server
    let req = format!("ws://{}", LISTEN).into_client_request().unwrap();
    let (mut ws_stream, _) = connect_async(req).await.unwrap();

    // send hello
    send(
        &mut ws_stream,
        ClientMsg::Hello {
            client_id: Uuid::default(),
            client_name: "Tester".into(),
        },
    )
    .await;

    let mut joined_instance = None;
    let mut lobby_joined = false;
    loop {
        let msg = recv(&mut ws_stream).await;
        match msg {
            ServerMsg::JoinedLobby {  } => {
                lobby_joined = true;
            },
            ServerMsg::Instances { instances } => {
                assert_eq!(lobby_joined, true);
                if joined_instance.is_none() {
                    assert_eq!(instances.len(), 10);
                    let first = instances.first().unwrap();
                    joined_instance = Some(first.clone());
                    send(&mut ws_stream, ClientMsg::JoinInstance { instance_id:first.id}).await;
                }
            },
            ServerMsg::JoinedInstance { instance } => {
                let joined_instance = joined_instance.as_ref().unwrap();
                assert_eq!(instance.id, joined_instance.id);
                send(&mut ws_stream, ClientMsg::CustomMsg { msg: [1,2,3,4].into() }).await;
            },
            ServerMsg::Pong { tick:_, server_bytes_sec:_, client_bytes_sec:_ } => {

            },
            ServerMsg::Custom { msg } => {
                assert_eq!(msg.len(), 4);
                break;
            },
            ServerMsg::JoinRejected {
                instance:_
            } => { }
        }
    }
}

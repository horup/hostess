use std::convert::TryFrom;

use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;
use crate::{Bincoded, UntypedContext, UntypedGameServer, UntypedGameMsg, UntypedHostMsg};


#[derive(Clone, Debug)]
pub enum HostMsg<T> {
    ClientJoined {
        client_id:Uuid
    },
    ClientLeft {
        client_id:Uuid
    },
    CustomMsg {
        client_id:Uuid,
        msg:T
    }
}

impl<T:Bincoded> TryFrom<UntypedHostMsg> for HostMsg<T> {
    type Error = ();

    fn try_from(hm: UntypedHostMsg) -> Result<Self, Self::Error> {
        match hm {
            UntypedHostMsg::ClientJoined { client_id } => {
                return Ok(HostMsg::ClientJoined {
                    client_id
                });
            },
            UntypedHostMsg::ClientLeft { client_id } => {
                return Ok(HostMsg::ClientLeft {
                    client_id
                });
            },
            UntypedHostMsg::CustomMsg { client_id, msg } => {
                
                if let Some(msg) = T::from_bincode(&msg) {
                    return Ok(HostMsg::CustomMsg {
                        client_id,
                        msg
                    });
                }
            },
        }

        Err(())
    }
}

#[derive(Clone, Debug)]
pub enum GameServerMsg<T> {
    CustomToAll {
        msg:T
    },
    CustomTo {
        client_id:Uuid,
        msg:T
    }
}


impl<T:Bincoded> TryFrom<GameServerMsg<T>> for UntypedGameMsg {
    type Error = ();

    fn try_from(value: GameServerMsg<T>) -> Result<Self, Self::Error> {
        match value {
            GameServerMsg::CustomToAll { 
                msg 
            } => {
                return Ok(UntypedGameMsg::CustomToAll {
                    msg: msg.to_bincode(),
                });
            },
            GameServerMsg::CustomTo { 
                client_id, msg 
            } => {
                return Ok(UntypedGameMsg::CustomTo {
                    client_id:client_id,
                    msg: msg.to_bincode(),
                });
            },
        }
    }

    
}

pub struct Context<T> {
    pub host_messages:Vec<HostMsg<T>>,
    pub game_messages:Vec<GameServerMsg<T>>
}

impl<T> Context<T> {
    pub fn new() -> Self {
        Context {
            host_messages: Vec::new(),
            game_messages: Vec::new(),
        }
    }
}
pub trait GameServer : UntypedGameServer {
    type CustomMsg : Serialize + DeserializeOwned + Bincoded;
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:&mut Context<Self::CustomMsg>);
}

impl<T: GameServer> UntypedGameServer for T {
    fn update(&mut self, context:&mut UntypedContext) {
        let mut c = Context::new();
        for msg in context.host_messages.drain(..) {
            if let Ok(msg) = HostMsg::<T::CustomMsg>::try_from(msg) {
                c.host_messages.push(msg);
            }
        }
        GameServer::update(self, &mut c);

        for msg in c.game_messages.drain(..) {
            if let Ok(msg) = UntypedGameMsg::try_from(msg) {
                context.game_messages.push_back(msg);
            }
        }
    }

    fn tick_rate(&self) -> u64 {
        GameServer::tick_rate(self)
    }
}
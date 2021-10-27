use std::convert::TryFrom;

use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;
use crate::{Bincoded, Context, Game, GameMsg, HostMsg};


#[derive(Clone, Debug)]
pub enum TypedHostMsg<T> {
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

impl<T:Bincoded> TryFrom<HostMsg> for TypedHostMsg<T> {
    type Error = ();

    fn try_from(hm: HostMsg) -> Result<Self, Self::Error> {
        match hm {
            HostMsg::ClientJoined { client_id } => {
                return Ok(TypedHostMsg::ClientJoined {
                    client_id
                });
            },
            HostMsg::ClientLeft { client_id } => {
                return Ok(TypedHostMsg::ClientLeft {
                    client_id
                });
            },
            HostMsg::CustomMsg { client_id, msg } => {
                
                if let Some(msg) = T::from_bincode(&msg) {
                    return Ok(TypedHostMsg::CustomMsg {
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
pub enum TypedGameMsg<T> {
    CustomToAll {
        msg:T
    },
    CustomTo {
        client_id:Uuid,
        msg:T
    }
}


impl<T:Bincoded> TryFrom<TypedGameMsg<T>> for GameMsg {
    type Error = ();

    fn try_from(value: TypedGameMsg<T>) -> Result<Self, Self::Error> {
        match value {
            TypedGameMsg::CustomToAll { 
                msg 
            } => {
                return Ok(GameMsg::CustomToAll {
                    msg: msg.to_bincode(),
                });
            },
            TypedGameMsg::CustomTo { 
                client_id, msg 
            } => {
                return Ok(GameMsg::CustomTo {
                    client_id:client_id,
                    msg: msg.to_bincode(),
                });
            },
        }
    }

    
}

pub struct TypedContext<A, B> {
    pub host_messages:Vec<TypedHostMsg<A>>,
    pub game_messages:Vec<TypedGameMsg<B>>
}

impl<A, B> TypedContext<A, B> {
    pub fn new() -> Self {
        TypedContext {
            host_messages: Vec::new(),
            game_messages: Vec::new(),
        }
    }
}
pub trait TypedGame : Game {
    type A : Serialize + DeserializeOwned + Bincoded;
    type B : Serialize + DeserializeOwned + Bincoded;
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:&mut TypedContext<Self::A, Self::B>);
}

impl<T: TypedGame> Game for T {
    fn update(&mut self, context:&mut Context) {
        let mut c = TypedContext::new();
        for msg in context.host_messages.drain(..) {
            if let Ok(msg) = TypedHostMsg::<T::A>::try_from(msg) {
                c.host_messages.push(msg);
            }
        }
        TypedGame::update(self, &mut c);

        for msg in c.game_messages.drain(..) {
            if let Ok(msg) = GameMsg::try_from(msg) {
                context.game_messages.push_back(msg);
            }
        }
    }

    fn tick_rate(&self) -> u64 {
        TypedGame::tick_rate(self)
    }
}
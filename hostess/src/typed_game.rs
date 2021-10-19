use std::convert::TryFrom;

use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;
use crate::{Bincoded, Context, Game, HostMsg};


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
   /* fn from(hm: HostMsg) -> Self {
        match hm {
            HostMsg::ClientJoined { client_id } => {
                return TypedHostMsg::ClientJoined {
                    client_id
                };
            },
            HostMsg::ClientLeft { client_id } => {
                return TypedHostMsg::ClientLeft {
                    client_id
                };
            },
            HostMsg::CustomMsg { client_id, msg } => {
                return TypedHostMsg::CustomMsg {
                    client_id,
                    msg:TypedHostMsg::
                }
            },
        }
    }*/
    
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
            /*match msg {
                crate::HostMsg::ClientJoined { client_id } => {
                    c.host_messages.push()
                },
                crate::HostMsg::ClientLeft { client_id } => todo!(),
                crate::HostMsg::CustomMsg { client_id, msg } => todo!(),
            }*/

            if let Ok(msg) = TypedHostMsg::<T::A>::try_from(msg) {
                c.host_messages.push(msg);
            }
        }
        TypedGame::update(self, &mut c);
    }

    fn tick_rate(&self) -> u64 {
        TypedGame::tick_rate(self)
    }
}
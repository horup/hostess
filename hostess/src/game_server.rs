use std::{convert::{TryFrom}, marker::PhantomData};

use uuid::Uuid;

use crate::{Bincoded, untyped_game_server::{UntypedContext, UntypedGameServer, UntypedGameServerMsg, UntypedHostMsg}};

#[derive(Clone, Debug)]
pub enum HostMsg<T> {
    ClientJoined {
        client_id:Uuid,
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

impl<T:Bincoded> From<GameServerMsg<T>> for UntypedGameServerMsg {
    fn from(msg: GameServerMsg<T>) -> Self {
        match msg {
            GameServerMsg::CustomToAll { 
                msg 
            } => {
                UntypedGameServerMsg::CustomToAll {
                    msg: msg.to_bincode(),
                }
            },
            GameServerMsg::CustomTo { 
                client_id, msg 
            } => {
                UntypedGameServerMsg::CustomTo {
                    client_id:client_id,
                    msg: msg.to_bincode(),
                }
            },
        }
    }
}

pub struct Context<T> {
    context:UntypedContext,
    phantom:PhantomData<T>
}

impl<T : Bincoded> Context<T> {
    pub fn lock(context:UntypedContext) -> Self {
        Self {
            context,
            phantom:PhantomData::default()
        }
    }

    pub fn release(self) -> UntypedContext {
        self.context
    }

    pub fn pop_host_msg(&mut self) -> Option<HostMsg<T>> {
        let msg = self.context.host_messages.pop_front();
        match msg {
            Some(msg) => {
                let msg = HostMsg::try_from(msg).ok();
                return msg;
            },
            None => return None,
        }
    }

    pub fn push_game_msg(&mut self, msg:GameServerMsg<T>) {
        let msg = msg.into();
        self.context.game_messages.push_back(msg);
    }
}

pub trait GameServer : UntypedGameServer {
    type CustomMsg : Bincoded;
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:&mut Context<Self::CustomMsg>);
}


impl<T: GameServer> UntypedGameServer for T {
    fn update(&mut self, mut context:UntypedContext) -> UntypedContext {
        let mut context = Context::lock(context);
        GameServer::update(self, &mut context);
        context.release()
    }

    fn tick_rate(&self) -> u64 {
        GameServer::tick_rate(self)
    }
}
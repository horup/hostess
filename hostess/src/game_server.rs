use std::{collections::VecDeque, sync::Arc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum HostMsg {
    ClientJoined {
        client_id:Uuid,
        client_name:String
    },
    ClientLeft {
        client_id:Uuid
    },
    CustomMsg {
        client_id:Uuid,
        msg:Vec<u8>
    }
}

#[derive(Clone, Debug)]
pub enum GameServerMsg {
    CustomToAll {
        msg:Vec<u8>
    },
    CustomTo {
        client_id:Uuid,
        msg:Vec<u8>
    }
}

pub struct Context {
    pub host_messages:VecDeque<HostMsg>,
    pub game_messages:VecDeque<GameServerMsg>,

    /// delta time between ticks in seconds between ticks
    /// this value can go from close zero to many thousands 
    /// and needs to be truncated or similar by the consumer to avoid
    /// unintended behavior, e.g. players jumping through walls due to high tick
    pub dt:f32
}

impl Context {
    pub fn pop_host_msg(&mut self) -> Option<HostMsg> {
        let msg = self.host_messages.pop_front();
        return msg;
    }

    pub fn push_game_msg(&mut self, msg:GameServerMsg) {
        let msg = msg.into();
        self.game_messages.push_back(msg);
    }
}

pub trait GameServer : Send + Sync + 'static {
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:Context) -> Context;
}

pub type GameServerConstructorFn = Box<dyn Fn() -> Box<dyn GameServer> + Send + Sync>;

#[derive(Clone)]
pub struct GameServerConstructor {
    arc:Arc<GameServerConstructorFn>
}

impl GameServerConstructor {
    pub fn new(f:GameServerConstructorFn) -> Self {
        Self {
            arc:Arc::new(Box::new(f))
        }
    }

    pub fn construct(&self) -> Box<dyn GameServer> {
        let f = self.arc.as_ref();
        f()
    }
}

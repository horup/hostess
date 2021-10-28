use std::{collections::VecDeque, sync::Arc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum UntypedHostMsg {
    ClientJoined {
        client_id:Uuid
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
pub enum UntypedGameMsg {
    CustomToAll {
        msg:Vec<u8>
    },
    CustomTo {
        client_id:Uuid,
        msg:Vec<u8>
    }
}

pub struct UntypedContext {
    pub host_messages:Vec<UntypedHostMsg>,
    pub game_messages:VecDeque<UntypedGameMsg>
}

pub trait UntypedGameServer : Send + Sync + 'static {
    //fn new() -> Self;
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:&mut UntypedContext);
}

//pub type GameConstructor = Arc<Box<dyn Fn() -> Box<dyn Game> + Send + Sync>>;

pub type GameServerConstructorFn = Box<dyn Fn() -> Box<dyn UntypedGameServer> + Send + Sync>;

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

    pub fn construct(&self) -> Box<dyn UntypedGameServer> {
        let f = self.arc.as_ref();
        f()
    }
}

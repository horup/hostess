use std::{collections::VecDeque, sync::Arc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum HostMsg {
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
pub enum GameMsg {
    CustomToAll {
        msg:Vec<u8>
    },
    CustomTo {
        client_id:Uuid,
        msg:Vec<u8>
    }
}

pub struct Context {
    pub host_messages:Vec<HostMsg>,
    pub game_messages:VecDeque<GameMsg>
}

pub trait Game : Send + Sync + 'static {
    //fn new() -> Self;
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:&mut Context);
}

//pub type GameConstructor = Arc<Box<dyn Fn() -> Box<dyn Game> + Send + Sync>>;

pub type GameConstructorFn = Box<dyn Fn() -> Box<dyn Game> + Send + Sync>;

#[derive(Clone)]
pub struct GameConstructor {
    arc:Arc<GameConstructorFn>
}

impl GameConstructor {
    pub fn new(f:GameConstructorFn) -> Self {
        Self {
            arc:Arc::new(Box::new(f))
        }
    }

    pub fn construct(&self) -> Box<dyn Game> {
        let f = self.arc.as_ref();
        f()
    }
}
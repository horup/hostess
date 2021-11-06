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
pub enum UntypedGameServerMsg {
    CustomToAll {
        msg:Vec<u8>
    },
    CustomTo {
        client_id:Uuid,
        msg:Vec<u8>
    }
}

pub struct UntypedContext {
    pub host_messages:VecDeque<UntypedHostMsg>,
    pub game_messages:VecDeque<UntypedGameServerMsg>
}

impl UntypedContext {
    pub fn pop_host_msg(&mut self) -> Option<UntypedHostMsg> {
        let msg = self.host_messages.pop_front();
        return msg;
    }

    pub fn push_game_msg(&mut self, msg:UntypedGameServerMsg) {
        let msg = msg.into();
        self.game_messages.push_back(msg);
    }
}

pub trait UntypedGameServer : Send + Sync + 'static {
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:UntypedContext) -> UntypedContext;
}

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

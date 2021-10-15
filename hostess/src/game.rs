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

}

pub struct Context {
    pub host_messages:Vec<HostMsg>,
    pub game_messages:Vec<GameMsg>
}

pub trait Game : Send + Sync + 'static {
    fn new() -> Self;
    fn tick_rate(&self) -> u64;
    fn update(&mut self, context:&mut Context);
}
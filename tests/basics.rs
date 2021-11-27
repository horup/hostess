use hostess::{game_server::{GameServer, GameServerConstructor}, server::Server};


#[derive(Default)]
pub struct Game {

}

impl GameServer for Game {
    fn tick_rate(&self) -> u64 {
        todo!()
    }

    fn tick(&mut self, context:hostess::game_server::Context) -> hostess::game_server::Context {
        todo!()
    }
}


#[test]
pub fn basics() {
    let hostess = Server::new("0.0.0.0:8080", GameServerConstructor::new::<Game>());

}
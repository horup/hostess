use hostess::{Context, Game, GameMsg};

pub struct Server {

}

impl Game for Server {
    fn new() -> Self {
        Self {
            
        }
    }

    fn tick_rate(&self) -> u64 {
        1
    }

    fn update(&mut self, context:&mut Context) {
        context.game_messages.push_back(GameMsg::CustomToAll{
            msg:[1,2,3,4,5,6,7,8].into()
        });
    }
}
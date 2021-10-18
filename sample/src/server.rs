use hostess::{Context, Game, GameMsg, log::info, Bincoded};
use sample_lib::{GameState, Msg};

pub struct Server {
    state:GameState
}

impl Server {
    pub fn new() -> Self {
        Self {
            state:GameState::demo()
        }
    }
}

impl Game for Server {
    fn tick_rate(&self) -> u64 {
        1
    }

    fn update(&mut self, context:&mut Context) {
        context.game_messages.push_back(GameMsg::CustomToAll{
            msg:Msg::SnapshotFull {
                state:self.state.clone()
            }.to_bincode()
        });

        info!("ticking... {}", context.host_messages.len());
    }
}
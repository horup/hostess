use crate::GameState;
use generational_arena::Index;
use glam::Vec2;
use hostess::Bincoded;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum GameServerMsg {
    SnapshotFull {
        state:GameState
    },
    PlayerThing {
        thing_id:Option<Index>
    }
}

#[derive(Serialize, Deserialize)]
pub enum GameClientMsg {
    ClientInput {
        position:Option<Vec2>,
        shoot:bool
    }
}

impl Bincoded for GameServerMsg {
    
}

impl Bincoded for GameClientMsg {
    
}
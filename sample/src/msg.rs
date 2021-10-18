use crate::GameState;
use glam::Vec2;
use hostess::Bincoded;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum GameServerMsg {
    SnapshotFull {
        state:GameState
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
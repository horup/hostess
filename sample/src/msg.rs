use crate::GameState;
use hostess::Bincoded;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Msg {
    SnapshotFull {
        state:GameState
    }
}

impl Bincoded for Msg {
    
}
use crate::{Input, State};
use generational_arena::Index;
use hostess::Bincoded;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum GameServerMsg {
    SnapshotFull {
        state:State
    },
    PlayerThing {
        thing_id:Option<Index>
    }
}

#[derive(Serialize, Deserialize)]
pub enum GameClientMsg {
    ClientInput {
        input:Input
    }
}

impl Bincoded for GameServerMsg {
    
}

impl Bincoded for GameClientMsg {
    
}
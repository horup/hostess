use std::convert::TryFrom;

use crate::{Input, State};
use generational_arena::Index;
use hostess::{Bincoded, GameMsg};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CustomMsg {
    SnapshotFull {
        state:State
    },
    PlayerThing {
        thing_id:Option<Index>
    },
    ClientInput {
        input:Input
    }
}


impl Bincoded for CustomMsg {
    
}

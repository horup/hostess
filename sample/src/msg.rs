use crate::{Input, State};
use generational_arena::Index;
use hostess::{Bincoded};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CustomMsg {
    ServerSnapshotFull {
        state:State
    },
    ServerPlayerThing {
        thing_id:Option<Index>
    },

    /// input from a client, such as position, ability usage, e.g.
    ClientInput {
        input:Input
    }
}


impl Bincoded for CustomMsg {
    
}

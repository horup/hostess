use crate::{Input, State};
use generational_arena::Index;
use hostess::{Bincoded};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CustomMsg {
    ServerSnapshotFull {
        /// the timestamp of the last input recv and processed by the server
        input_timestamp_sec:f64,
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

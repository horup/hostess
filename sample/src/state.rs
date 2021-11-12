use generational_arena::{Arena};
use serde::{Deserialize, Serialize};

use crate::{Thing};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub things: Arena<Thing>,
    pub width: f32,
    pub height: f32,
}

impl State {
    pub fn new() -> Self {
        Self {
            things: Arena::new(),
            width: 40.0,
            height: 30.0,
        }
    }
}

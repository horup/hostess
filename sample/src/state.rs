use std::collections::VecDeque;

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::log::info;
use serde::{Deserialize, Serialize};

use crate::{Input, LocalChange, Thing};

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

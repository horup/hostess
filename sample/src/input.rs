use std::collections::VecDeque;

use generational_arena::Index;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// struct holding Input for a player
/// send by clients to the server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    /// the timestamp of the input
    pub timestamp_sec:f64,

    /// the id of the thing controlled by a player owning the Input
    pub thing_id:Option<Index>,

    /// direction of the thing according to what the player believes is true
    pub movement_dir:Vec2,

    /// position of the thing according to what the player believes is true
    pub pos:Vec2,

    /// true if the player wants to use his ability
    pub ability_trigger:bool,

    /// where the player is targeting in the world
    pub ability_target:Vec2,

    #[serde(skip)]
    pub local_changes:Vec<LocalChange>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalChange {
    pub timestamp_sec:f64,
    pub v:Vec2
}

impl Default for Input {
    fn default() -> Self {
        Self { 
            timestamp_sec:0.0,
            thing_id: Default::default(), 
            movement_dir: Default::default(), 
            pos: Default::default(), 
            ability_trigger: Default::default(), 
            ability_target: Default::default(),
            local_changes: Default::default()
        }
    }
}
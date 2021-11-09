use std::collections::VecDeque;

use generational_arena::Index;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::Change;


/// struct holding Input for a player
/// send by clients to the server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    /// the id of the thing controlled by a player owning the Input
    pub thing_id:Option<Index>,

    /// direction of the thing according to what the player believes is true
    pub movement_dir:Vec2,

    /// position of the thing according to what the player believes is true
    //pub pos:Vec2,

    /// true if the player wants to use his ability
    pub ability_activated:bool,

    /// where the player is targeting in the world
    pub target_pos:Vec2,

    /// changes to gamestate since last recv from server
    pub changes:VecDeque<Change>
}
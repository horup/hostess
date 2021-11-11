use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Thing {
    /// position of the thing
    pub pos:Vec2,

    /// change of position
    pub vel:Vec2,

    /// the radius of the thing
    pub radius:f32,

    /// direction where the thing points
    /// not neccesarily equal to the velocity
    pub dir:f32,

    /// health of the thing, zero or less equals dead
    pub health:f32,

    /// cooldown of ability
    /// zero indicates the ability is ready
    pub ability_cooldown:f32,

    /// true if this is a player
    pub is_player:bool,

    /// true if this is a projectile
    pub is_projectile:bool,

    /// name of the thing
    pub name:String,

    /// max speed of thing
    pub max_speed:f32
}




impl Thing {
    pub fn new_player(x:f32, y:f32) -> Self {
        Self {
            pos:[x, y].into(),
            vel:[0.0, 0.0].into(),
            radius:0.5,
            dir:0.0,
            health:100.0,
            ability_cooldown:0.0,
            name:"".into(),
            is_player:true,
            max_speed:5.0,
            ..Default::default()
        }
    }

    pub fn new_projectile(pos:Vec2, vel:Vec2) -> Self {
        Self {
            pos,
            vel,
            radius:0.25,
            dir:0.0,
            health:100.0,
            is_projectile:true,
            max_speed:10.0,
            ..Default::default()
        }
    }

    pub fn random_new_player(state:&State) -> Self {
        let thing = Thing::new_player(rand::random::<f32>() * state.width, rand::random::<f32>() * state.height);
        thing
    }
}
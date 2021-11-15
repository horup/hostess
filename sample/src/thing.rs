use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Solid {
    /// not solid, does not influence its own movement nor others
    None,

    /// solid, but influences only its own movement, not others, i.e. others ignore it
    Partial,

    /// solid, all movement is influenced by this
    Solid
}

impl Default for Solid {
    fn default() -> Self {
        Self::Solid
    }
}

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

    pub solid:Solid,

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
    pub speed:f32,

    pub respawn_timer:f32,

    #[serde(skip)]
    pub no_interpolate:bool
}




impl Thing {
    pub fn new_player(x:f32, y:f32) -> Self {
        Self {
            pos:[x, y].into(),
            vel:[0.0, 0.0].into(),
            radius:0.5,
            dir:0.0,
            health:1.0,
            ability_cooldown:0.0,
            name:"".into(),
            is_player:true,
            speed:5.0,
            ..Default::default()
        }
    }

    pub fn respawn(&mut self, x:f32, y:f32) {
        self.pos = Vec2::new(x, y);
        self.solid = Solid::Solid;
        self.health = 1.0;
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn new_projectile(pos:Vec2, vel:Vec2) -> Self {
        Self {
            pos,
            vel,
            radius:0.25,
            dir:0.0,
            health:100.0,
            is_projectile:true,
            speed:10.0,
            solid:Solid::Partial,
            ..Default::default()
        }
    }

    pub fn random_new_player(state:&State) -> Self {
        let thing = Thing::new_player(rand::random::<f32>() * state.width, rand::random::<f32>() * state.height);
        thing
    }

    pub fn lerp_pos(&self, prev:&Thing, alpha:f32) -> Vec2 {
        let pos = self.pos;
        let v = pos - prev.pos;
        if v.length() < 2.0 {
            let v = v * alpha;
            return pos + v;
        }

        return pos;
    }
}
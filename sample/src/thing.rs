use generational_arena::Index;
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
pub struct PlayerThing {
    pub health:f32,
    pub respawn_timer:f32,
    pub ability_cooldown:f32,
    pub speed:f32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectileThing {
    pub vel:Vec2,
    pub owner:Index
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Specialization {
    None,
    Player(PlayerThing),
    Projectile(ProjectileThing)
}

impl Default for Specialization {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Thing {
    /// position of the thing
    pub pos:Vec2,

    /// the radius of the thing
    pub radius:f32,

    pub solid:Solid,

    /// name of the thing
    pub name:String,

    pub specialization:Specialization,

    #[serde(skip)]
    pub no_interpolate:bool
}




impl Thing {
    pub fn new_player(x:f32, y:f32) -> Self {
        Self {
            pos:[x, y].into(),
            radius:0.5,
            specialization:Specialization::Player(PlayerThing {
                health:1.0,
                speed:5.0,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn as_player_mut(&mut self) -> Option<&mut PlayerThing> {
        if let Specialization::Player(player) = &mut self.specialization {
            return Some(player);
        }
        None
    }

    pub fn as_player(&self) -> Option<&PlayerThing> {
        if let Specialization::Player(player) = &self.specialization {
            return Some(player);
        }
        None
    }

    pub fn as_projectile_mut(&mut self) -> Option<&mut ProjectileThing> {
        if let Specialization::Projectile(projectile) = &mut self.specialization {
            return Some(projectile);
        }
        None
    }

    pub fn as_projectile(&self) -> Option<&ProjectileThing> {
        if let Specialization::Projectile(projectile) = &self.specialization {
            return Some(projectile);
        }
        None
    }

    pub fn respawn(&mut self, x:f32, y:f32) {
        self.pos = Vec2::new(x, y);
        self.solid = Solid::Solid;
        if let Some(player) = self.as_player_mut() {
            player.health = 1.0;
            player.respawn_timer = 0.0;
        }
    }

    pub fn new_projectile(pos:Vec2, vel:Vec2, owner:Index) -> Self {
        Self {
            pos,
            radius:0.25,
            solid:Solid::Partial,
            specialization:Specialization::Projectile(ProjectileThing {
                vel,
                owner
            }),
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
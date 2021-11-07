use generational_arena::{Arena, Index};
use glam::Vec2;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Thing {
    /// position of the thing
    pub pos:Vec2,

    /// velocity of the thing
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

    /// name of the thing
    pub name:String,
}


impl Thing {
    pub fn new(x:f32, y:f32) -> Self {
        Self {
            pos:[x, y].into(),
            vel:[0.0, 0.0].into(),
            radius:0.5,
            dir:0.0,
            health:100.0,
            ability_cooldown:0.0,
            name:"".into(),
            is_player:true
        }
    }

    pub fn random_new(state:&State) -> Self {
        let thing = Thing::new(rand::random::<f32>() * state.width, rand::random::<f32>() * state.height);
        thing
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub things:Arena<Thing>,
    pub width:f32,
    pub height:f32
}

/// struct holding Input for a player
/// send by clients to the server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    /// the id of the thing controlled by a player owning the Input
    pub thing_id:Option<Index>,


    /// direction of the thing according to what the player believes is true
    pub movement_dir:Vec2,

    /// position of the thing according to what the player believes is true
    pub pos:Vec2,

    /// true if the player wants to use his ability
    pub ability_activated:bool,

    /// where the player is targeting in the world
    pub target_pos:Vec2
}

impl State {
    pub fn new() -> Self
    {
        Self {
            things:Arena::new(),
            width:40.0,
            height:30.0
        }
    }

    pub fn update(&mut self, input:Option<&mut Input>, dt:f64) {
        if let Some(input) = input {
            if let Some(thing_id) = input.thing_id {
                if let Some(thing) = self.things.get_mut(thing_id) {
                    let speed = 1.0 * dt;
                    thing.pos.y += input.movement_dir.y * speed as f32;
                    thing.pos.x += input.movement_dir.x * speed as f32;

                    input.pos = thing.pos;
                }
            }
        }
    }
}
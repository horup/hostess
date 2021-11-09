use std::collections::VecDeque;

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::log::info;
use serde::{Serialize, Deserialize};

use crate::{Input, Thing};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub things:Arena<Thing>,
    pub width:f32,
    pub height:f32
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

    pub fn reapply_input(&mut self, input:&mut Input) {
        if let Some(thing_id) = input.thing_id {
            if let Some(thing) = self.things.get_mut(thing_id) {
               
            }
        }
    }

    pub fn update(&mut self, input:Option<&mut Input>, dt:f64) {
        if let Some(input) = input {
            if let Some(thing_id) = input.thing_id {
                if let Some(thing) = self.things.get_mut(thing_id) {
                    let mut v = Vec2::new(input.movement_dir.x * thing.max_speed * dt as f32, input.movement_dir.y * thing.max_speed * dt as f32);
                    if v.length() > thing.max_speed {
                        v = v.normalize() * thing.max_speed;
                    }

                    thing.pos += v;
                }
            }
        }
    }
}
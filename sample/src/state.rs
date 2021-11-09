use std::collections::VecDeque;

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::log::info;
use serde::{Serialize, Deserialize};

use crate::{Input, Thing};



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub timestamp_sec:f64,
    pub things:Arena<Thing>,
    pub width:f32,
    pub height:f32
}


/// struct holding the changes since last state change
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Change {
    pub timestamp_sec:f64,
    pub v:Vec2
}


impl State {
    pub fn new() -> Self
    {
        Self {
            timestamp_sec:0.0,
            things:Arena::new(),
            width:40.0,
            height:30.0
        }
    }

    pub fn reapply_input(&mut self, input:&mut Input) {
        if let Some(thing_id) = input.thing_id {
            if let Some(thing) = self.things.get_mut(thing_id) {
                while let Some(front) = input.changes.front() {
                    if front.timestamp_sec < self.timestamp_sec {
                        input.changes.pop_front();
                    } else {
                        break;
                    }
                }

                for change in &input.changes {
                    if change.timestamp_sec > self.timestamp_sec {
                        thing.pos += change.v;
                    }
                }
            }
        }
    }

    pub fn update(&mut self, input:Option<&mut Input>, dt:f64) {
        self.timestamp_sec += dt;
        if let Some(input) = input {
            if let Some(thing_id) = input.thing_id {
                if let Some(thing) = self.things.get_mut(thing_id) {
                    let mut v = Vec2::new(input.movement_dir.x * thing.max_speed * dt as f32, input.movement_dir.y * thing.max_speed * dt as f32);
                    if v.length() > thing.max_speed {
                        v = v.normalize() * thing.max_speed;
                    }

                    input.changes.push_back(Change {
                        timestamp_sec: self.timestamp_sec,
                        v,
                    });
                    thing.pos += v;
                }
            }
        }
    }
}
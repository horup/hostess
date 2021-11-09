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

    pub fn reapply_input(&mut self, input: &mut Input, timestamp_sec: f64) {
        let iter = input
            .local_changes
            .drain(..)
            .filter(|x| x.timestamp_sec > timestamp_sec);
        input.local_changes = iter.collect();

        if let Some(thing_id) = input.thing_id {
            if let Some(thing) = self.things.get_mut(thing_id) {
                let diff = input.pos - thing.pos;
                for c in &input.local_changes {
                    thing.pos += c.v;
                }
            }
        }
    }

    pub fn update(&mut self, input: Option<&mut Input>, dt: f64) {
        if let Some(input) = input {
            if let Some(thing_id) = input.thing_id {
                if let Some(thing) = self.things.get_mut(thing_id) {
                    let mut v = Vec2::new(
                        input.movement_dir.x * thing.max_speed * dt as f32,
                        input.movement_dir.y * thing.max_speed * dt as f32,
                    );
                    if v.length() > thing.max_speed {
                        v = v.normalize() * thing.max_speed;
                    }

                    let change = LocalChange {
                        timestamp_sec: input.timestamp_sec,
                        v: v,
                    };

                    input.local_changes.push(change);

                    thing.pos += v;
                    input.pos = thing.pos;
                }
            }
        }
    }
}

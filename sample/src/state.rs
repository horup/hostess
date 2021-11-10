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

    /// returns tuple if collision occured
    /// with whom and new position
    pub fn simple_collision_test(thing_id:&Index, thing:&mut Thing, candidates:&Arena<Thing>) -> Option<(Vec2, Index)> {
        for (test_id, test) in candidates.iter() {
            if test_id != *thing_id {
                let v = thing.pos - test.pos;
                let l = test.radius + thing.radius;
                if v.length() < l && v.length() != 0.0 {
                    let l = l - v.length();
                    let mut pos = thing.pos;
                    let v = v.normalize() * l;
                    pos += v;
                    thing.pos = pos;
                    return Some((pos, test_id));
                }
            }
        }

        None
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

        for (id, thing) in self.things.iter_mut() {
            thing.pos += thing.vel * dt as f32;

            let mut outta_bounds = false;
            if thing.pos.x < 0.0 + thing.radius {
                thing.pos.x = 0.0 + thing.radius;
                outta_bounds = true;
            } else if thing.pos.x > self.width - thing.radius {
                thing.pos.x = self.width - thing.radius;
                outta_bounds = true;
            }

            if thing.pos.y < 0.0 + thing.radius {
                thing.pos.y = 0.0 + thing.radius;
                outta_bounds = true;
            } else if thing.pos.y > self.height - thing.radius {
                thing.pos.y = self.height - thing.radius;
                outta_bounds = true;
            }

            if outta_bounds && thing.is_projectile {
                //thing.health = 0.0;
            }
        }

        let candidates = self.things.clone();
        for (id, thing) in self.things.iter_mut() {
            Self::simple_collision_test(&id, thing, &candidates);
        }



    }
}

use std::collections::HashMap;

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::uuid::Uuid;

use crate::{Input, LocalChange, Player, State, Thing};


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

/*
#[derive(Default)]
pub struct Simulator {
    spawn:Vec<Thing>,
    remove:Vec<Index>
}

impl Simulator {
    fn clear(&mut self) {
        self.spawn.clear();
        self.remove.clear();
    }

    pub fn server_update(&mut self, state:&mut State, players:&HashMap<Uuid, Player>, dt: f64) {
        self.clear();
        
        self.shared_physics(state, dt);
        //self.shared_update(state, None, dt);
    }

    pub fn client_update(&mut self, state:&mut State, input:&mut Input, dt:f64) {
        self.clear();

        // process input locally
        if let Some(thing_id) = input.thing_id {
            if let Some(thing) = state.things.get_mut(thing_id) {
                let mut v = glam::Vec2::new(
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

                thing.vel += v;
                input.pos = thing.pos;
            }
        }
        


        self.shared_physics(state, dt);
        //self.shared_update(state, Some(input), dt);
    }

    pub fn reapply_input(&mut self, state:&mut State, input: &mut Input, timestamp_sec: f64) {
      /*  let iter = input
            .local_changes
            .drain(..)
            .filter(|x| x.timestamp_sec > timestamp_sec);
        input.local_changes = iter.collect();

        if let Some(thing_id) = input.thing_id {
            if let Some(thing) = state.things.get_mut(thing_id) {
                let diff = input.pos - thing.pos;
                for c in &input.local_changes {
                    thing.pos += c.v;
                }
            }
        }*/
    }

    pub fn shared_physics(&mut self, state:&mut State, dt:f64) {
        for (thing_id, thing) in &mut state.things {
            
            // do movement and collision detection
            thing.pos += thing.vel * dt as f32;

             // bounds check
          /*  let mut outta_bounds = false;
            if thing.pos.x < 0.0 + thing.radius {
                thing.pos.x = 0.0 + thing.radius;
                outta_bounds = true;
            } else if thing.pos.x > state.width - thing.radius {
                thing.pos.x = state.width - thing.radius;
                outta_bounds = true;
            }

            if thing.pos.y < 0.0 + thing.radius {
                thing.pos.y = 0.0 + thing.radius;
                outta_bounds = true;
            } else if thing.pos.y > state.height - thing.radius {
                thing.pos.y = state.height - thing.radius;
                outta_bounds = true;
            }

            if outta_bounds && thing.is_projectile {
                //thing.health = 0.0;
                self.remove.push(id);
            }*/
        }

       
    }

    pub fn shared_update2(&mut self, state:&mut State, input: Option<&mut Input>, dt: f64) {
        if let Some(input) = input {
            if let Some(thing_id) = input.thing_id {
                if let Some(thing) = state.things.get_mut(thing_id) {
                    let mut v = glam::Vec2::new(
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

                    //input.local_changes.push(change);
                    thing.pos += v;
                    input.pos = thing.pos;
                }
            }
        } else {
            // server only

            for (id, thing) in state.things.iter_mut() {
                thing.pos += thing.vel * dt as f32;
    
                let mut outta_bounds = false;
                if thing.pos.x < 0.0 + thing.radius {
                    thing.pos.x = 0.0 + thing.radius;
                    outta_bounds = true;
                } else if thing.pos.x > state.width - thing.radius {
                    thing.pos.x = state.width - thing.radius;
                    outta_bounds = true;
                }
    
                if thing.pos.y < 0.0 + thing.radius {
                    thing.pos.y = 0.0 + thing.radius;
                    outta_bounds = true;
                } else if thing.pos.y > state.height - thing.radius {
                    thing.pos.y = state.height - thing.radius;
                    outta_bounds = true;
                }
    
                if outta_bounds && thing.is_projectile {
                    //thing.health = 0.0;
                    self.remove.push(id);
                }

                thing.ability_cooldown -= dt as f32;
                if thing.ability_cooldown < 0.0 {
                    thing.ability_cooldown = 0.0;
                }
    
                if thing.is_player && thing.ability_trigger && thing.ability_cooldown <= 0.0 {
                    thing.ability_cooldown = 0.25;
                    let dir = thing.ability_target - thing.pos;
                    if dir.length() > 0.0 {
                        let v = dir.normalize() * 10.0;
                        let p = Thing::new_projectile(thing.pos, v);
                        self.spawn.push(p);
                    }
                }
            }

            for thing_id in self.remove.drain(..) {
                state.things.remove(thing_id);
            }

            for thing in self.spawn.drain(..) {
                state.things.insert(thing);
            }
        }

        let candidates = state.things.clone();
        for (id, thing) in state.things.iter_mut() {
            simple_collision_test(&id, thing, &candidates);
        }
    }
}


*/


pub fn apply_input(state:&mut State, input:&Input) {
    if let Some(thing_id) = input.thing_id {
        if let Some(thing) = state.things.get_mut(thing_id) {
            thing.pos += input.movement * thing.max_speed as f32;
        }
    }
}
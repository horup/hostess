
use generational_arena::{Arena, Index};
use glam::Vec2;

use crate::{Input, Player, State, Thing};

pub fn apply_input(state:&mut State, input:&Input, authorative:bool) {
    let mut spawn = Vec::new();
    // how to avoid clone?
    let cloned = state.clone();
    if let Some(thing_id) = input.thing_id {
        if let Some(thing) = state.things.get_mut(thing_id) {
            let new_pos = thing.pos + input.movement * thing.speed as f32;
            move_thing_y_then_x((thing_id, thing), new_pos, &cloned);

            if authorative {
                if input.ability_trigger && thing.ability_cooldown <= 0.0 {
                    thing.ability_cooldown = 0.25;
                    let dir = input.ability_target - thing.pos;
                    if dir.length() > 0.0 {
                        let dir = dir.normalize();
                        let v = dir * 20.0;
                        let p = Thing::new_projectile(thing.pos + dir, v);
                        spawn.push(p);
                    }
                }
            }
        }
    }

    for thing in spawn.drain(..) {
        state.things.insert(thing);
    }
}

pub fn update_things(state:&mut State, dt:f64) {
    // how to avoid clone
    let cloned = state.clone();

    let mut remove = Vec::new();
    let mut hits = Vec::new();

    for (id, thing)  in state.things.iter_mut() {
        thing.ability_cooldown -= dt as f32;
        if thing.ability_cooldown < 0.0 {
            thing.ability_cooldown = 0.0;
        }

        if thing.vel.length_squared() > 0.0 {
            let new_pos = thing.pos + thing.vel * dt as f32;
            let res = move_thing_y_then_x((id, thing), new_pos, &cloned);
            if let CollisionResult::Thing(hit_id) = res {
                if thing.is_projectile {
                    remove.push(id);
                    hits.push(hit_id);
                }
            }
        }
    }

    for id in remove.drain(..) {
        state.things.remove(id);
    }

    for id in hits.drain(..) {
        if let Some(thing) = state.things.get_mut(id) {
            thing.health -= 1.0;
        }
    }
}

pub struct Circle {
    pub c:Vec2,
    pub r:f32
}


/// performs a test between two circles
fn collision_test_circle_circle(circle1:Circle, circle2:Circle) -> bool {
    let d = circle1.c - circle2.c;
    if d.length() > 0.0 {
        let l = circle1.r + circle2.r;
        let l = d.length() - l;
        if l < 0.0 {
            return true;
        }
    }
    
    false
}

#[derive(PartialEq, Eq)]
pub enum CollisionResult {
    None,
    Thing(Index)
}

/// move the thing while avoiding collisions, first in y then x
pub fn move_thing_y_then_x(thing:(Index, &mut Thing), new_pos:Vec2, state:&State) -> CollisionResult {
    let (thing_id, thing1) = thing;
    let pos = Vec2::new(thing1.pos.x, new_pos.y);
    
    let res1 = move_thing_direct((thing_id, thing1), pos, state);
    let pos = Vec2::new(new_pos.x, thing1.pos.y);
    let res2 = move_thing_direct((thing_id, thing1), pos, state);

    if res1 != CollisionResult::None {
        return res1;
    } else {
        return res2;
    }
}

/// move the thing while avoiding collisions
fn move_thing_direct(thing:(Index, &mut Thing), new_pos:Vec2, state:&State) -> CollisionResult {
    let (thing_id, thing1) = thing;
    let mut result = CollisionResult::None;
    for (thing_id2, thing2) in state.things.iter() {
        if thing_id != thing_id2 {
            let dir = new_pos - thing1.pos;
            let n = thing1.pos - thing2.pos;
            let dir = dir.normalize();
            let n = n.normalize();

            if dir.dot(n) < 0.0 {
                let hit = collision_test_circle_circle(Circle {
                    c:new_pos,
                    r:thing1.radius
                }, Circle {
                    c:thing2.pos,
                    r:thing2.radius
                });
                if hit {
                    result = CollisionResult::Thing(thing_id2);
                    break;
                }
            }
        }
    }

    if result == CollisionResult::None {
        thing1.pos = new_pos;
    }

    return result;
}

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::log::info;

use crate::{Event, Input, Player, Solid, State, Thing};

pub fn apply_input(state:&mut State, input:&Input, authorative:bool) {
    // how to avoid clone?
    let cloned = state.clone();
    if let Some(thing_id) = input.thing_id {
        if let Some(thing) = state.things.get_mut(thing_id) {
            let mut new_pos = thing.pos;
            if let Some(player) = thing.as_player_mut() {
                if player.health > 0.0 {
                    new_pos = input.movement * player.speed as f32 + thing.pos;
                    move_thing_y_then_x((thing_id, thing), new_pos, &cloned);
                }
            }
        }
    }
}

pub fn update_things(state:&mut State, dt:f64) {
    // how to avoid clone
    let cloned = state.clone();

    let mut remove = Vec::new();
    let mut hits = Vec::new();

    // movement and collision handling
    for (id, thing)  in state.things.iter_mut() {
        if let Some(player) = thing.as_player_mut() {
            player.ability_cooldown -= dt as f32;
            if player.ability_cooldown < 0.0 {
                player.ability_cooldown = 0.0;
            }
        }

        if let Some(projectile) = thing.as_projectile_mut() {
            if projectile.vel.length_squared() > 0.0 {
                let new_pos = projectile.vel * dt as f32 + thing.pos;
                let res = move_thing_y_then_x((id, thing), new_pos, &cloned);
                if let CollisionResult::Thing(hit_id) = res {
                    remove.push(id);
                    hits.push(hit_id);
                }
            }
        }
       
    }

    // hit / damage handling
    for id in hits.drain(..) {
        if let Some(thing) = state.things.get_mut(id) {
            if let Some(player) = thing.as_player_mut() {
                if player.health > 0.0 {
                    player.health -= 1.0;
    
                    if player.health <= 0.0 {
                        player.respawn_timer = 3.0;
                        thing.solid = Solid::None;

                        state.events.push(Event::PlayerDied {
                            thing_id:id,
                            pos:thing.pos
                        });
                    }
                }
            }
        }
    }

    // player respawn handling
    for (id, thing) in state.things.iter_mut() {
        if let Some(player) = thing.as_player_mut() {
            if player.health <= 0.0 {
                player.respawn_timer -= dt as f32;
                if player.respawn_timer <= 0.0 {
                    player.respawn_timer = 0.0;
                    thing.respawn(rand::random::<f32>() * state.width, rand::random::<f32>() * state.height);
                }

            }
        }
    }

    // ensuring things stay within bounds
    // and remove projectiles that venture out of bounds
    let w = state.width;
    let h = state.height;
    for (id, thing) in state.things.iter_mut() {
        let pos = thing.pos;
        thing.pos = pos.clamp(Vec2::new(0.0, 0.0), Vec2::new(w,h));
        
        if let Some(projectile) = thing.as_projectile_mut() {
            if pos != thing.pos {
                remove.push(id);

            }
        }
    }
    
    // removal of entities who needs removed
    for id in remove.drain(..) {
        state.things.remove(id);
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

    if thing1.solid != Solid::None {
        for (thing_id2, thing2) in state.things.iter() {
            if thing2.solid == Solid::Solid {
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
        }
    }

    if result == CollisionResult::None {
        thing1.pos = new_pos;
    }

    return result;
}
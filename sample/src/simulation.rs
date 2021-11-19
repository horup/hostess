
use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::log::info;
use parry2d::{math::Isometry, query::{contact, details::contact_ball_ball}, shape::Ball};

use crate::{Event, Input, Player, Solid, State, Thing};

pub fn apply_input(state:&mut State, input:&Input, authorative:bool) {
    // how to avoid clone?
    let cloned = state.clone();
    if let Some(thing_id) = input.thing_id {
        if let Some(thing) = state.things.get_mut(thing_id) {
            if let Thing::Player(player) = thing {
                let mut new_pos = player.pos;
                if player.is_alive() {
                    new_pos = input.movement * player.speed as f32 + *thing.pos();
                    move_thing_direct((thing_id, thing), new_pos, &cloned, None);
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
        if let Thing::Player(player) = thing {
            player.ability_cooldown -= dt as f32;
            if player.ability_cooldown < 0.0 {
                player.ability_cooldown = 0.0;
            }
        }

        if let Thing::Projectile(projectile) = thing {
            let owner = projectile.owner;
            if projectile.vel.length_squared() > 0.0 {
                let new_pos = projectile.vel * dt as f32 + *thing.pos();
                let res = move_thing_direct((id, thing), new_pos, &cloned, Some(owner));
                if let CollisionResult::Thing(target) = res {
                    remove.push(id);
                    hits.push((owner, target));
                }
            }
        }
       
    }

    // hit / damage handling
    for (owner, target) in hits.drain(..) {
        if let Some(thing) = state.things.get_mut(target) {
            if let Thing::Player(player) = thing {
                if player.is_alive() {
                    player.hearts -= 1;
    
                    if !player.is_alive() {
                        player.respawn_timer = 3.0;
                        player.deaths += 1;
                        player.solid = Solid::None;

                        state.events.push(Event::PlayerDied {
                            thing_id:target,
                            pos:player.pos
                        });

                        if let Some(thing) = state.things.get_mut(owner) {
                            if let Thing::Player(owner) = thing {
                                owner.kills += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // player respawn handling
    for (id, thing) in state.things.iter_mut() {
        if let Thing::Player(player) = thing {
            if !player.is_alive() {
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
        let pos = *thing.pos();
        *thing.pos_mut() = pos.clamp(Vec2::new(0.0, 0.0), Vec2::new(w,h));
        
        if let Thing::Projectile(projectile) = thing {
            if pos != projectile.pos {
                remove.push(id);

            }
        }
    }
    
    // removal of entities who needs removed
    for id in remove.drain(..) {
        if let Some(thing) = state.things.remove(id) {
            if let Thing::Projectile(projectile) = thing {
                state.events.push(Event::ProjectileHit{
                    pos: projectile.pos,
                })
            }
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
/*
/// move the thing while avoiding collisions, first in y then x
pub fn move_thing_y_then_x(thing:(Index, &mut Thing), new_pos:Vec2, state:&State, ignore:Option<Index>) -> CollisionResult {
  /*  let (thing_id, thing1) = thing;
    let pos = Vec2::new(thing1.pos().x, new_pos.y);
    let res1 = move_thing_direct((thing_id, thing1), pos, state, ignore);
    let pos = Vec2::new(new_pos.x, thing1.pos().y);
    let res2 = move_thing_direct((thing_id, thing1), pos, state, ignore);

    if res1 != CollisionResult::None {
        return res1;
    } else {
        return res2;
    }*/

    move_thing_direct(thing, new_pos, state, ignore)
}*/

/// move the thing while avoiding collisions
pub fn move_thing_direct(thing:(Index, &mut Thing), new_pos:Vec2, state:&State, ignore:Option<Index>) -> CollisionResult {
    let mut result = CollisionResult::None;
    let (thing_id, thing1) = thing;
    *thing1.pos_mut() = new_pos;
    if *thing1.solid() != Solid::None {
        for (thing_id2, thing2) in state.things.iter() {
            // check same
            if thing_id == thing_id2 {
                continue;
            }

            // check if Solid and not just partially solid
            if *thing2.solid() != Solid::Solid {
                continue;
            }

            // check ignore
            if let Some(ignore) = ignore {
                if thing_id2 == ignore {
                    continue;
                }
            }

            let pos1:Isometry<f32> = [thing1.pos().x, thing1.pos().y].into();
            let ball1 = Ball::new(*thing1.radius());
            
            let pos2:Isometry<f32> = [thing2.pos().x, thing2.pos().y].into();
            let ball2 = Ball::new(*thing2.radius());

            let c = contact(&pos1, &ball1, &pos2, &ball2, 1.0);
            match c {
                Ok(res) => {
                    match res {
                        Some(res) => {
                            if res.dist < 0.0 {
                                let p:Vec2 = [res.normal1.x, res.normal1.y].into();
                                let p = p * res.dist;
                                *thing1.pos_mut() += p;
                                result = CollisionResult::Thing(thing_id2);
                                break;
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }
    }
  
    return result;
}
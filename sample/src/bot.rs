use generational_arena::Index;
use glam::Vec2;
use hostess::log::info;
use sample_lib::{State, move_thing_y_then_x};

pub struct Bot {
    pub thing_id:Index
}

impl Bot {
    pub fn tick(&mut self, state:&mut State, delta:f64) {
        // how to avoid clone?
        let cloned = state.clone();
        if let Some(thing) = state.things.get_mut(self.thing_id) {
            if thing.health > 0.0 {
                let v = Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5).normalize();
                let v = v * thing.speed * delta as f32;
                let new_pos = thing.pos + v ;
                move_thing_y_then_x((self.thing_id, thing), new_pos, &cloned);
            }
        }
    }
}
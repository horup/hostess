use generational_arena::Arena;
use glam::Vec2;

pub struct Thing {
    pub pos:Vec2,
    pub vel:Vec2,
    pub radius:f32,
    pub name:String
}

impl Thing {
    pub fn new(x:f32, y:f32) -> Self {
        Self {
            pos:[x, y].into(),
            vel:[0.0, 0.0].into(),
            radius:0.5,
            name:"".into()
        }
    }
}


pub struct GameState {
    pub things:Arena<Thing>
}

impl GameState {
    pub fn new() -> Self
    {
        Self {
            things:Arena::new()
        }
    }

    pub fn demo() -> Self {
        let mut state = Self::new();
        
        // make some players
        for i in 0..10 {
            let mut thing = Thing::new(rand::random::<f32>() * 40.0, rand::random::<f32>() * 30.0);
            thing.name = format!("P{}", i);
            state.things.insert(thing);
        }

        state
    }
}
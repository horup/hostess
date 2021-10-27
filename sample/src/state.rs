use generational_arena::{Arena, Index};
use glam::Vec2;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn random_new(state:&State) -> Self {
        let thing = Thing::new(rand::random::<f32>() * state.width, rand::random::<f32>() * state.height);
        thing
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub things:Arena<Thing>,
    pub width:f32,
    pub height:f32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub thing_id:Option<Index>,
    pub dir:Vec2,
    pub position:Vec2,
    pub shoot:bool
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

    pub fn demo() -> Self {
        let mut state = Self::new();
        
        // make some players
        for i in 0..10 {
            let mut thing = Thing::new(rand::random::<f32>() * state.width, rand::random::<f32>() * state.height);
            thing.name = format!("P{}", i);
            state.things.insert(thing);
        }

        state
    }

    pub fn update(&mut self, input:Option<&mut Input>) {
        if let Some(input) = input {
            if let Some(thing_id) = input.thing_id {
                if let Some(thing) = self.things.get_mut(thing_id) {
                    let speed = 0.1;
                    thing.pos.y += input.dir.y * speed;
                    thing.pos.x += input.dir.x * speed;

                    input.position = thing.pos;
                }
            }
        }
    }
}
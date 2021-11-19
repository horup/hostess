use std::collections::VecDeque;

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::Bincoded;
use serde::{Deserialize, Serialize};

use crate::Thing;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    PlayerDied {
        thing_id:Index,
        pos:Vec2
    },
    ProjectileHit {
        pos:Vec2
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Polygon {
    pub points:Vec<Vec2>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub timestamp:f64,
    pub things: Arena<Thing>,
    pub events:Vec<Event>,
    pub map:Arena<Polygon>,
    pub width: f32,
    pub height: f32,
}

impl Bincoded for State {
}

impl State {
    pub fn new() -> Self {
        let mut map = Arena::new();
        map.insert(Polygon {
            points:[Vec2::new(7.0, 7.0), Vec2::new(10.0, 7.0), Vec2::new(9.0, 13.0)].into()
        });
        Self {
            timestamp:0.0,
            things: Arena::new(),
            width: 40.0,
            height: 30.0,
            events:Vec::new(),
            map
        }
    }
}

pub struct StateHistory {
    history:VecDeque<State>,
    default_state:State
}

impl StateHistory {
    pub fn new() -> Self {
        StateHistory {
            history:VecDeque::with_capacity(10),
            default_state:State::new(),
        }
    }

    pub fn remember(&mut self, state:State) {
        if self.history.len() > 20 {
            self.history.pop_front();
        }

        self.history.push_back(state);
    }

    pub fn len(&self) -> usize {
        return self.history.len();
    }
    pub fn current(&self) -> &State {
        if let Some(newest) = self.history.back() {
            return newest;
        }
            
        &self.default_state
    }

    pub fn prev(&self) -> &State {
        if let Some(s) = self.history.get(self.len() - 2) {
            return s;
        }

        &self.default_state
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}
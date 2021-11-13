use std::collections::VecDeque;

use generational_arena::{Arena, Index};
use glam::Vec2;
use hostess::Bincoded;
use serde::{Deserialize, Serialize};

use crate::Thing;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub things: Arena<Thing>,
    pub width: f32,
    pub height: f32,
}

impl Bincoded for State {
}

impl State {
    pub fn new() -> Self {
        Self {
            things: Arena::new(),
            width: 40.0,
            height: 30.0,
        }
    }
/*
    pub fn mutate(&mut self, commands:&Commands) {
        for command in commands.iter() {
            match command {
                Command::SetThing { thing_id, thing } => {
                    if let Some(t) = self.things.get_mut(*thing_id) {
                        *t = thing.clone();
                    }
                },
                Command::MoveThing { thing_id, pos } => {
                    if let Some(thing) = self.things.get_mut(*thing_id) {
                        thing.pos = *pos;
                    }
                },
                Command::ApplyDamageToThing { thing_id, amount } => todo!(),
                Command::RemoveThing { thing_id } => {
                    self.things.remove(*thing_id);
                },
                Command::SetThings {
                    things
                } => {
                    self.things = things.clone();
                }
            }
        }
    }*/
}


pub struct StateHistory {
    history:VecDeque<State>,
    empty_state:State
}

impl StateHistory {
    pub fn new() -> Self {
        StateHistory {
            history:VecDeque::with_capacity(10),
            empty_state:State::new()
        }
    }

    pub fn remember(&mut self, state:State) {
        if self.history.len() > 20 {
            self.history.pop_front();
        }

        self.history.push_back(state);
    }

    pub fn last(&self) -> &State {
        if let Some(last) = self.history.back() {
            return last;
        }
            
        &self.empty_state
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}


/*
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Command {
    SetThings {
        things:Arena<Thing>
    },
    SetThing {
        thing_id:Index,
        thing:Thing
    },
    MoveThing {
        thing_id:Index,
        pos:Vec2
    },
    ApplyDamageToThing {
        thing_id:Index,
        amount:f32 
    },
    RemoveThing {
        thing_id:Index,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    pub fn new() -> Commands {
        Commands {
            commands: Vec::with_capacity(1024),
        }
    }

    pub fn push(&mut self, command:Command) {
        self.commands.push(command);
    }

    pub fn iter(&self) -> core::slice::Iter<Command> {
        self.commands.iter()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

*/
use std::collections::HashMap;
use generational_arena::Index;
use glam::Vec2;
use hostess::{Bincoded, ClientMsg, Context, Game, GameMsg, log::info, typed_game::{TypedGame, TypedGameMsg, TypedHostMsg}, uuid::Uuid};
use sample_lib::{GameClientMsg, GameServerMsg, State, Thing};
use serde::{Serialize, Deserialize};
use web_sys::console::info;


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub client_id:Uuid,
    pub thing:Option<Index>
}

pub struct Server {
    state:State,
    players:HashMap<Uuid, Player>
}

impl Server {
    pub fn new() -> Self {
        Self {
            state:State::new(),
            players:HashMap::new()
        }
    }
}

impl TypedGame for Server {
    type A = GameClientMsg;
    type B = GameServerMsg;

    fn update(&mut self, context:&mut hostess::typed_game::TypedContext<Self::A, Self::B>) {
        info!("ticking");
        for msg in context.host_messages.drain(..) {
            match msg {
                hostess::typed_game::TypedHostMsg::ClientJoined { client_id } => {
                    if !self.players.contains_key(&client_id) {
                        self.players.insert(client_id, Player {
                            client_id:client_id,
                            thing:None
                        });
                    }
                },
                hostess::typed_game::TypedHostMsg::ClientLeft { client_id } => {

                },
                hostess::typed_game::TypedHostMsg::CustomMsg { client_id, msg } => {
                    match msg {
                        GameClientMsg::ClientInput { input } => {
                            if let Some(player) = self.players.get_mut(&client_id) {
                                if input.shoot && player.thing == None {
                                    // spawn player thing
                                    let thing = Thing::random_new(&self.state);
                                    player.thing = Some(self.state.things.insert(thing));

                                   /* context.game_messages.push_back(GameServerMsg::CustomTo {
                                        client_id:player.client_id,
                                        msg:GameServerMsg::PlayerThing {
                                            thing_id:player.thing
                                        }.to_bincode()
                                    });*/

                                    context.game_messages.push(TypedGameMsg::CustomTo {
                                        client_id:player.client_id,
                                        msg:GameServerMsg::PlayerThing {
                                            thing_id:player.thing
                                        }
                                    });
                                }

                                if let Some(thing_id) = player.thing {
                                    if let Some(thing) = self.state.things.get_mut(thing_id) {
                                        thing.pos = input.position;
                                    }
                                }
                            }


                        },
                    }
                },
            }
        }

        context.game_messages.push(TypedGameMsg::CustomToAll {
            msg:GameServerMsg::SnapshotFull {
                state:self.state.clone()
            }
        });
    }

    fn tick_rate(&self) -> u64 {
        2
    }
}

/*
impl Game for Server {
    fn tick_rate(&self) -> u64 {
        20
    }

    fn update(&mut self, context:&mut Context) {
        for msg in context.host_messages.drain(..) {
            match msg {
                hostess::HostMsg::ClientJoined { client_id } => {
                    if !self.players.contains_key(&client_id) {
                        self.players.insert(client_id, Player {
                            client_id:client_id,
                            thing:None
                        });
                    }
                    
                },
                hostess::HostMsg::ClientLeft { client_id } => {
                },
                hostess::HostMsg::CustomMsg { client_id, msg } => {
                    if let Some(msg) = GameClientMsg::from_bincode(&msg) {
                        match msg {
                            GameClientMsg::ClientInput { 
                                input 
                            } => {
                                if let Some(player) = self.players.get_mut(&client_id) {
                                    if input.shoot && player.thing == None {
                                        // spawn player thing
                                        let thing = Thing::random_new(&self.state);
                                        player.thing = Some(self.state.things.insert(thing));

                                        context.game_messages.push_back(GameMsg::CustomTo {
                                            client_id:player.client_id,
                                            msg:GameServerMsg::PlayerThing {
                                                thing_id:player.thing
                                            }.to_bincode()
                                        });
                                    }

                                    if let Some(thing_id) = player.thing {
                                        if let Some(thing) = self.state.things.get_mut(thing_id) {
                                            thing.pos = input.position;
                                        }
                                    }
                                }
                            },
                        }
                    }
                },
            }
        }
        context.game_messages.push_back(GameMsg::CustomToAll{
            msg:GameServerMsg::SnapshotFull {
                state:self.state.clone()
            }.to_bincode()
        });
    }
} */ 
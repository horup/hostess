use std::collections::HashMap;
use generational_arena::Index;
use glam::Vec2;
use hostess::{Bincoded, ClientMsg, log::info, game_server::{Context, GameServer, GameServerMsg, HostMsg}, uuid::Uuid};
use sample_lib::{CustomMsg, State, Thing};
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

impl GameServer for Server {
    fn tick_rate(&self) -> u64 {
        todo!()
    }

    fn update(&mut self, mut context:Context) -> Context {

        while let Some(msg) = context.pop_host_msg() {
            match msg {
                HostMsg::ClientJoined { client_id } => {
                    if !self.players.contains_key(&client_id) {
                        self.players.insert(client_id, Player {
                            client_id:client_id,
                            thing:None
                        });
                    }
                },
                HostMsg::ClientLeft { client_id } => {

                },
                HostMsg::CustomMsg { client_id, msg } => {
                    if let Some(msg) = Bincoded::from_bincode(&msg) {
                        self.custom_msg(&mut context, client_id, msg);
                    }
                },
            }
        }

        return context;
    }
}

impl Server {
    pub fn push_custom_to(context:&mut Context, client_id:Uuid, msg:CustomMsg) {
        let msg = msg.to_bincode();
        context.push_game_msg(GameServerMsg::CustomTo {
            client_id,
            msg
        });
    }

    pub fn custom_msg(&mut self, context:&mut Context, client_id:Uuid, msg:CustomMsg) {
        match msg {
            CustomMsg::ClientInput { input } => {
                if let Some(player) = self.players.get_mut(&client_id) {
                    if input.shoot && player.thing == None {
                        // spawn player thing
                        let thing = Thing::random_new(&self.state);
                        player.thing = Some(self.state.things.insert(thing));

                        Self::push_custom_to( context, player.client_id, CustomMsg::ServerPlayerThing {
                                thing_id:player.thing
                        });
                    }

                    if let Some(thing_id) = player.thing {
                        if let Some(thing) = self.state.things.get_mut(thing_id) {
                            thing.pos = input.position;
                        }
                    }
                }


            },
            CustomMsg::ServerSnapshotFull { state } => {},
            CustomMsg::ServerPlayerThing { thing_id } => {},
        }
    }
}
/*
impl GameServer for Server {
    type CustomMsg = CustomMsg;

    fn update(&mut self, context:&mut hostess::game_server::Context<CustomMsg>) {
        while let Some(msg) = context.pop_host_msg() {
            match msg {
                hostess::game_server::HostMsg::ClientJoined { client_id } => {
                    if !self.players.contains_key(&client_id) {
                        self.players.insert(client_id, Player {
                            client_id:client_id,
                            thing:None
                        });
                    }
                },
                hostess::game_server::HostMsg::ClientLeft { client_id } => {

                },
                hostess::game_server::HostMsg::CustomMsg { client_id, msg } => {
                    match msg {
                        CustomMsg::ClientInput { input } => {
                            if let Some(player) = self.players.get_mut(&client_id) {
                                if input.shoot && player.thing == None {
                                    // spawn player thing
                                    let thing = Thing::random_new(&self.state);
                                    player.thing = Some(self.state.things.insert(thing));

                                    context.push_game_msg(GameServerMsg::CustomTo {
                                        client_id:player.client_id,
                                        msg:CustomMsg::ServerPlayerThing {
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
                        CustomMsg::ServerSnapshotFull { state } => {},
                        CustomMsg::ServerPlayerThing { thing_id } => {},
                    }
                },
            }
        }

        context.push_game_msg(GameServerMsg::CustomToAll {
            msg:CustomMsg::ServerSnapshotFull {
                state:self.state.clone()
            }
        });
    }

    fn tick_rate(&self) -> u64 {
        2
    }
}*/

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
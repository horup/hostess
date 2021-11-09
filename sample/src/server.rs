use std::collections::HashMap;
use generational_arena::Index;
use glam::Vec2;
use hostess::{Bincoded, log::info, game_server::{Context, GameServer, GameServerMsg, HostMsg}, uuid::Uuid};
use sample_lib::{CustomMsg, Input, State, Thing};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub client_id:Uuid,
    pub client_name:String,
    pub thing:Option<Index>,
    pub input:Input
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
        20
    }

    fn tick(&mut self, mut context:Context) -> Context {
        while let Some(msg) = context.pop_host_msg() {
            match msg {
                HostMsg::ClientJoined { client_id, client_name } => {
                    if !self.players.contains_key(&client_id) {
                        self.players.insert(client_id, Player {
                            client_id:client_id,
                            client_name,
                            thing:None,
                            input:Input::default()
                        });
                    }
                },
                HostMsg::ClientLeft { client_id } => {
                    if let Some(player) = self.players.remove(&client_id) {
                        if let Some(thing_id) = player.thing {
                            self.state.things.remove(thing_id);
                        }
                    }
                },
                HostMsg::CustomMsg { client_id, msg } => {
                    if let Some(msg) = Bincoded::from_bincode(&msg) {
                        self.recv_custom_msg(&mut context, client_id, msg);
                    }
                },
            }
        }


        self.state.update(None, context.delta);

        
        for (client_id, player) in &self.players {
            push_custom_to(&mut context, *client_id, CustomMsg::ServerSnapshotFull {
                input_timestamp_sec:player.input.timestamp_sec,
                state:self.state.clone()
            });
        }

        return context;
    }
}

fn push_custom_all(context:&mut Context, msg:CustomMsg) {
    let msg = msg.to_bincode();
    context.push_game_msg(GameServerMsg::CustomToAll {
        msg
    });
}
fn push_custom_to(context:&mut Context, client_id:Uuid, msg:CustomMsg) {
    let msg = msg.to_bincode();
    context.push_game_msg(GameServerMsg::CustomTo {
        client_id,
        msg
    });
}

impl Server {
    /// is called on each custom message received from the clients
    pub fn recv_custom_msg(&mut self, context:&mut Context, client_id:Uuid, msg:CustomMsg) {
        match msg {
            CustomMsg::ClientInput { input } => {
                if let Some(player) = self.players.get_mut(&client_id) {
                    if player.thing == None {
                        // player has no thing
                        let mut thing = Thing::random_new(&self.state);
                        thing.name = player.client_name.clone();
                        player.thing = Some(self.state.things.insert(thing));

                        // push state update to player
                        push_custom_to(context, player.client_id, CustomMsg::ServerSnapshotFull {
                            state:self.state.clone(),
                            input_timestamp_sec:player.input.timestamp_sec
                        });

                        // let the player know his thing id
                        push_custom_to(context, player.client_id, CustomMsg::ServerPlayerThing {
                            thing_id:player.thing
                        });
                    }

                    if let Some(thing_id) = player.thing {
                        if let Some(thing) = self.state.things.get_mut(thing_id) {
                            let mut v = input.pos - thing.pos;
                            if v.length() > thing.max_speed * context.delta as f32 {
                                v = v.normalize() * thing.max_speed * context.delta as f32; 
                            }

                            thing.pos += v;
                        }
                    }

                    // remember last recv input
                    player.input = input;
                }


            },
            _ => {}
        }
    }
}
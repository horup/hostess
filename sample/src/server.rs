use std::collections::HashMap;
use generational_arena::Index;
use glam::Vec2;
use hostess::{Bincoded, ClientMsg, Context, Game, GameMsg, log::info, uuid::Uuid};
use sample_lib::{GameClientMsg, GameServerMsg, GameState, Thing};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub client_id:Uuid,
    pub thing:Option<Index>
}

pub struct Server {
    state:GameState,
    players:HashMap<Uuid, Player>
}

impl Server {
    pub fn new() -> Self {
        Self {
            state:GameState::new(),
            players:HashMap::new()
        }
    }
}

impl Game for Server {
    fn tick_rate(&self) -> u64 {
        1
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
                                position, 
                                shoot 
                            } => {
                                if let Some(player) = self.players.get_mut(&client_id) {
                                    if shoot && player.thing == None {
                                        let thing = Thing {
                                            name:"".into(),
                                            pos:Vec2::new(2.0, 2.0),
                                            radius:0.5,
                                            vel:Vec2::new(0.0, 0.0)
                                        };
                                        player.thing = Some(self.state.things.insert(thing));
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

        info!("ticking... {}", context.host_messages.len());
    }
}
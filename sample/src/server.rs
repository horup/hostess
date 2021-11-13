use std::{collections::{HashMap, VecDeque}, ops::IndexMut};
use hostess::{Bincoded, log::info, game_server::{Context, GameServer, GameServerMsg, HostMsg}, uuid::Uuid};
use sample_lib::{Command, Commands, CustomMsg, Input, Player, State, Thing, apply_input2, update_things};
use serde::{Serialize, Deserialize};
use crate::bot::*;

pub struct Server {
    state:State,
    players:HashMap<Uuid, Player>,
    bots:Vec<Bot>
}

impl Server {
    pub fn new() -> Self {
       
        Self {
            state:State::new(),
            players:HashMap::new(),
            bots:Vec::new()
        }
    }

    pub fn update(&mut self, context:&mut Context) {
        let mut commands = Commands::new();
        if self.players.len() < 2 {
            // less than two players and no bots, ensure 10 bots are spawned
            if self.bots.len() == 0 {
                while self.bots.len() < 10 {
                    let mut thing = Thing::random_new_player(&self.state);
                    thing.name = "bot".into();
                    let index = self.state.things.insert(thing);
                    let bot = Bot {
                        thing_id:index
                    };
    
                    self.bots.push(bot);
                }

                // fix dont allow mutation above directly
                commands.push(Command::SetThings {
                    things:self.state.things.clone()
                });
            }
        } else {
            // more than two players, remove bots and their things
            for bot in self.bots.drain(..) {
                //self.state.things.remove(bot.thing_id);
                commands.push(Command::RemoveThing {
                    thing_id:bot.thing_id
                });
            }
        }

        // process inputs from players
        for (_, player) in &mut self.players {
            // if player has no 'thing'
            // ensure one is spawned for the player
            if player.thing == None {
                let mut thing = Thing::random_new_player(&self.state);
                thing.name = player.client_name.clone();
                player.thing = Some(self.state.things.insert(thing));

                // let the player know his thing id
                push_custom_to(context, player.client_id, CustomMsg::ServerPlayerThing {
                    thing_id:player.thing
                });

                // fix dont allow mutation above
                commands.push(Command::SetThings {
                    things:self.state.things.clone()
                });
            }

            // apply input from players
            for input in player.inputs.drain(..) {
                player.latest_input_timestamp_sec = input.timestamp_sec;
                apply_input2(&self.state, &mut commands, &input, true);
            }
        }

        // process bots
      /*  for bot in self.bots.iter_mut() {
            bot.tick(&mut self.state, context.delta);
        }

        update_things(&mut self.state, context.delta);
*/
        // update state
        self.state.mutate(&commands);

        // for each player, transmit Commands to them
        for (client_id, player) in &self.players {
            push_custom_to(context, *client_id, CustomMsg::ServerCommands {
                input_timestamp_sec:player.latest_input_timestamp_sec,
                commands:commands.clone()
            });
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
                            inputs:VecDeque::default(),
                            latest_input_timestamp_sec: 0.0,
                        });
                    }

                    push_custom_to(&mut context, client_id, CustomMsg::ServerSnapshotFull {
                        input_timestamp_sec:0.0,
                        state:self.state.clone()
                    });
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

        self.update(&mut context);

       

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
                    /*if player.thing == None {
                        // player has no thing
                        let mut thing = Thing::random_new_player(&self.state);
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
                            thing.ability_target = input.ability_target;
                            thing.ability_trigger = input.ability_trigger;
                        }
                    }*/

                    // remember last recv input
                    player.inputs.push_back(input);
                }


            },
            _ => {}
        }
    }
}
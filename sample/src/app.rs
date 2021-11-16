
use std::collections::VecDeque;

use generational_arena::Arena;
use glam::Vec2;
use hostess::{Bincoded, ClientMsg, ServerMsg, log::info, uuid::Uuid};
use crate::{CustomMsg, Input, State, StateHistory, Thing, apply_input, get_item, performance_now_ms, player, set_item, update_things};
use super::Canvas;

pub struct App {
    player_name:String,
    debug:bool,
    app_state:AppState,
    id:Uuid,
    canvas:Canvas,
    current:State,
    history:StateHistory,
    connection_status:String,
    ping:f64,
    client_bytes_sec:f32,
    server_bytes_sec:f32,
    input:Input,
    input_history:VecDeque<Input>,
    updates:u64,
    server_tick_rate:u8,
    since_last_snapshot_sec:f32,
    lerp_alpha:f32,
    effects:Arena<Effect>,
    pub server_messages:Vec<ServerMsg>,
    pub client_messages:Vec<ClientMsg>
}

pub type KeyCode = u32;


#[derive(Clone, PartialEq)]
/// enum holdning the client app state
enum AppState {
    /// the initial state
    /// whenever the client app is first connected or reconnected
    /// the initial state will be this
    Initial,

    /// enter name state where the player can enter his/her name before joing the match
    /// maybe this should be part of the hostess protocol
    EnterName {
        name:String
    },

    /// player is ready to join
    ReadyToJoin,

    /// when in game and playing
    InGame
}

#[derive(Clone)]
struct Effect {
    pub pos:Vec2,
    pub time:f32,
    pub vel:Vec2,
    pub radius:f32,
}



impl App {
    pub fn new() -> Self {
        Self {
            player_name:get_item("player_name").unwrap_or_default(),
            debug:true,
            app_state:AppState::Initial,
            canvas:Canvas::new(),
            current:State::new(),
            input:Input::default(),
            input_history:VecDeque::new(),
            server_messages:Vec::new(),
            connection_status:"Not connected!".into(),
            client_messages:Vec::new(),
            id:Uuid::new_v4(),
            ping:0.0,
            server_bytes_sec:0.0,
            client_bytes_sec:0.0,
            updates:0,
            history:StateHistory::new(),
            server_tick_rate:64,
            since_last_snapshot_sec:0.0,
            lerp_alpha:0.0,
            effects:Arena::new()
        }
    }

    pub fn init(&mut self) {
        self.canvas.set_image_src(0, "dummy.png");
    }

    pub fn draw(&self) {
        self.canvas.clear();
        let grid_size = 16.0;
        self.canvas.set_scale(grid_size);

        if self.app_state == AppState::InGame {
            self.draw_game();
        }

        let cx = (self.canvas.width() / grid_size as u32 / 2) as f64;
        let cy =  (self.canvas.height() / grid_size as u32 / 2) as f64;

      
        self.draw_ui_gameui();
        self.draw_ui_debug(grid_size);
        self.draw_ui_centercontent(cx, cy);
    }

    fn draw_effect(&self, effect:&Effect) {
        self.canvas.draw_circle(effect.pos.x as f64, effect.pos.y as f64, effect.radius as f64);
    }

    fn draw_thing(&self, thing:&Thing, pos:Vec2) {
        let x = pos.x as f64;
        let y = pos.y as f64;
        if let Some(player) = thing.as_player() {
            if player.health <= 0.0 {
                return;
            }
        } 
        
        self.canvas.draw_circle(x, y, thing.radius as f64);
    }

    fn draw_thing_name(&self, thing:&Thing, pos:Vec2) {
        let x = pos.x as f64;
        let y = pos.y as f64;
        if let Some(player) = thing.as_player() {
            if player.health <= 0.0 {
                return;
            }
        } 
        if thing.name.len() > 0 {
            self.canvas.fill_text(&thing.name, x, y - 1.0);
        }
    }

    fn draw_game(&self) {
        if self.app_state != AppState::InGame {
            return;
        }

        for (id, thing) in &self.current.things {
            if thing.no_interpolate {
                self.draw_thing(thing, thing.pos);
                continue;
            }

            if let Some(prev) = self.history.prev().things.get(id) {
                self.draw_thing(thing, thing.lerp_pos(prev, self.lerp_alpha));
            }
        }
        for (id, thing) in &self.current.things {
            if thing.no_interpolate {
                self.draw_thing_name(thing, thing.pos);
                continue;
            }
            
            if let Some(prev) = self.history.prev().things.get(id) {
                self.draw_thing_name(thing, thing.lerp_pos(prev, self.lerp_alpha));
            }
        }

        for (id, effect) in self.effects.iter() {
            self.draw_effect(effect);
        }
    }

    fn draw_ui_gameui(&self) {
        if self.app_state != AppState::InGame {
            return;
        }

        self.canvas.set_text_style("left", "middle");

        if let Some(thing_id) = self.input.thing_id {
            if let Some(thing) = self.current.things.get(thing_id) {
                if let Some(player) = thing.as_player() {
                    self.canvas.fill_text(format!("{:0.00}%", player.health).as_str(), 0.0, 0.5);
                }
            }
        }

    }

    fn draw_ui_centercontent(&self, cx: f64, cy: f64) {
        self.canvas.set_text_style("center", "middle");
        match &self.app_state {
            AppState::Initial | AppState::ReadyToJoin => {
                self.canvas.fill_text(&self.connection_status, cx, cy);
            },
            AppState::EnterName { name } => {
                self.canvas.fill_text(&format!("please enter player name:"), cx, cy);
                let name:String = name.clone() + if self.updates % 60 > 30 {"|".into()} else {" ".into()};

                self.canvas.fill_text(name.as_str(), cx, cy + 1.0);
            },
            AppState::InGame {  } => {
                if let Some(thing_id) = self.input.thing_id {
                    if let Some(thing) = self.current.things.get(thing_id) {
                        if let Some(player) = thing.as_player() {
                            if player.health <= 0.0 {
                                self.canvas.fill_text(&format!("You are dead! Respawning... {:0.00}", player.respawn_timer), cx, cy);
                            }
                        }
                       
                    }
                }
            },
        };
    }

    fn draw_ui_debug(&self, grid_size: f64) {
        if self.debug {
            self.canvas.set_text_style("right", "middle");
            self.canvas.fill_text(format!("{:0.00} ms", self.ping).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 0.5);
            self.canvas.fill_text(format!("{:.3} KiB/s", self.server_bytes_sec / 1024.0).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 1.5);
            //self.canvas.fill_text(format!("recv:{:0.00} kb/s", self.server_bytes_sec / 1000.0).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 2.5);
        }
    }

    pub fn send(&mut self, msg:ClientMsg) {
        self.client_messages.push(msg)
    }

    pub fn recv_custom(&mut self, msg:CustomMsg) {
        match msg {
            CustomMsg::ServerSnapshotFull { state,  input_timestamp_sec } => {
                self.since_last_snapshot_sec = 0.0;
                self.history.remember(state.clone());
                self.current = state;
                let inputs = self.input_history.clone();
                self.input_history.clear();
                for input in inputs { 
                    if input.timestamp_sec > input_timestamp_sec {
                        apply_input(&mut self.current, &input, false);
                        self.input_history.push_back(input);
                    }
                }
            },
            CustomMsg::ServerSnapshotDelta {
                delta,
                input_timestamp_sec
            } => {
                let state = State::from_delta_bincode(self.history.current(), &delta);
                let state = state.expect("Failed to deserialize from delta!");
                self.recv_custom(CustomMsg::ServerSnapshotFull {
                        input_timestamp_sec,
                        state
                    }
                );
            },
            CustomMsg::ServerPlayerInfo {
                thing_id,
                tick_rate
            } => {
                self.server_tick_rate = tick_rate;
                self.input.thing_id = thing_id;
                if let Some(thing_id) = thing_id {
                    if let Some(thing) = self.current.things.get(thing_id) {
                        self.input.movement = thing.pos.clone();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn recv(&mut self, msg:&ServerMsg) {
        match msg {
            ServerMsg::LobbyJoined {  } => {
                self.connection_status = "Connected to Server".into();
               
            },
            ServerMsg::Hosts {hosts} => {
                if let Some(host) = hosts.first() {
                    self.connection_status = format!("Joining host {}..", host.id);
                    let id = host.id;
                    self.send(ClientMsg::JoinHost {
                        host_id:id
                    });
                }
            },
            ServerMsg::HostJoined {host} => {
                self.connection_status = format!("✓ Joined host {} ✓ ", host.id);
                self.new_app_state(AppState::InGame);
            },
            ServerMsg::Pong {
                tick,
                client_bytes_sec,
                server_bytes_sec
            } => {
                let ping:f64 = performance_now_ms() - tick;
                self.ping = ping;
                self.server_bytes_sec = *server_bytes_sec;
                self.client_bytes_sec = *client_bytes_sec;
            },
            ServerMsg::Custom { msg } => {
                let msg = CustomMsg::from_bincode(msg).unwrap();
                self.recv_custom(msg);
            }
            _ => {}
        }
    }

    pub fn send_custom(&mut self, msg:CustomMsg) {
        self.client_messages.push(ClientMsg::CustomMsg {
            msg:msg.to_bincode()
        });
    }

    pub fn update(&mut self, dt:f64) {
        // process messages
        self.since_last_snapshot_sec += dt as f32;
        for msg in &self.server_messages.clone() {
            self.recv(msg);
        }

        // calculate lerp which is used to do smooth linear interpolation between things
        self.lerp_alpha = self.since_last_snapshot_sec / (1.0 / self.server_tick_rate as f32);

        // ping server every 60 update
        if self.updates % 60 == 0 {
            self.send(ClientMsg::Ping {
                tick:performance_now_ms()
            });
        }

        // calculate movement and apply to local thing
        // update input with timestamp and movement data, and send to server
        self.input.timestamp_sec = performance_now_ms() / 1000.0;
        self.input.movement = self.input.movement_dir * dt as f32;

        // remember input for later processing
        self.input_history.push_back(self.input.clone());

        // apply input now
        apply_input(&mut self.current, &self.input, false);


        // send input to server
        self.send_custom(CustomMsg::ClientInput {
            input:self.input.clone()
        });

        // ensure player is not interpolated
        if let Some(thing_id) = self.input.thing_id {
            if let Some(thing) = self.current.things.get_mut(thing_id) {
                thing.no_interpolate = true;
            }
        }

        // process events
        for e in self.current.events.drain(..) {
            info!("{:?}", e);
        }


        let mut clean = Vec::new();
        // update effects
        for (id, effect) in self.effects.iter_mut() {
            effect.time -= dt as f32;
            if effect.time < 0.0 {
                clean.push(id);
            }
        }
        clean.drain(..).for_each(|id| {self.effects.remove(id);});
        
        for (id, thing) in self.current.things.iter() {
            if let Some(projectile) = thing.as_projectile() {
                self.effects.insert(Effect {
                    pos: thing.pos,
                    time: 1.0,
                    vel: Vec2::new(0.0, 0.0),
                    radius: thing.radius,
                });
            }
        }

        // draw some stuff
        self.draw();
        self.updates += 1; 
    }

    pub fn keyup(&mut self, code:KeyCode, _key:&str) {
        match &self.app_state {
            AppState::InGame {  } => {
                let i = &mut self.input;
                if code == 87 && i.movement_dir.y == -1.0 {
                    i.movement_dir.y = 0.0;
                }
                if code == 83 && i.movement_dir.y == 1.0 {
                    i.movement_dir.y = 0.0;
                }
                if code == 65 && i.movement_dir.x == -1.0 {
                    i.movement_dir.x = 0.0;
                }
                if code == 68 && i.movement_dir.x == 1.0 {
                    i.movement_dir.x = 0.0;
                }

                if code == 32 {
                    i.ability_trigger = false;
                }
            },
            _ => {

            }
        };
    }

    pub fn keydown(&mut self, code:KeyCode, key:&str) {
        match &mut self.app_state {
            AppState::EnterName { name } => {
                if key.is_ascii() && name.len() < 16 && key.len() == 1 {
                    *name += key;
                }
                else if key == "Enter" && name.len() > 0 {
                    self.player_name = name.clone();
                    set_item("player_name", self.player_name.as_str());
                    self.new_app_state(AppState::ReadyToJoin {});
                }
                else if key == "Backspace" && name.len() > 0 {
                    *name = name[0..name.len()-1].into();
                }
            },
            AppState::InGame {  } => {
                let i = &mut self.input;
                if code == 87 {
                    i.movement_dir.y = -1.0;
                }
                if code == 83 {
                    i.movement_dir.y = 1.0;
                }
        
                if code == 65 {
                    i.movement_dir.x = -1.0;
                }
                if code == 68 {
                    i.movement_dir.x = 1.0;
                }
        
                if code == 32 {
                    i.ability_trigger = true;
                }
            },
            _ => {

            }
        };

        // w = 87
        // s = 83
        // a = 65
        // d = 68
        // space = 32
        // up = 38
        // down = 40
        // left = 37
        // right = 39
        // esc = 27
    }

    pub fn mousemove(&mut self, x:f32, y:f32) {
        self.input.ability_target = Vec2::new(x / 16.0, y / 16.0);
    }

    pub fn mousedown(&mut self, button:u32, x:f32, y:f32) {
        self.input.ability_target = Vec2::new(x / 16.0, y / 16.0);
        if button == 0 {
            self.input.ability_trigger = true;
        }
    }

    pub fn mouseup(&mut self, button:u32, x:f32, y:f32) {
        self.input.ability_target = Vec2::new(x, y);
        if button == 0 {
            self.input.ability_trigger = false;
        }
    }

    fn new_app_state(&mut self, new_app_state:AppState) {
        self.app_state = new_app_state;
        match &self.app_state {
            AppState::ReadyToJoin => {
                self.connection_status = format!("Sending Hello");
                self.client_messages.push(ClientMsg::Hello {
                    client_id:self.id.clone(),
                    client_name:self.player_name.clone()
                });
            }
            _ => {},
        }
    }

    pub fn connected(&mut self) {
        self.history.clear();
        self.current = State::new();
        self.connection_status = format!("Connected");
        self.new_app_state(AppState::EnterName {
            name:self.player_name.clone()
        });
    }

    pub fn disconnected(&mut self) {
        self.connection_status = "Trying to reconnect...".into();
        self.new_app_state(AppState::Initial);
    }
}

unsafe impl Send for App {
}
unsafe impl Sync for App {
}

use glam::Vec2;
use hostess::{Bincoded, ClientMsg, ServerMsg, log::info, uuid::Uuid};
use crate::{CustomMsg, Input, State, get_item, performance_now, set_item};
use super::Canvas;

pub struct App {
    player_name:String,
    debug:bool,
    app_state:AppState,
    id:Uuid,
    canvas:Canvas,
    state:State,
    connection_status:String,
    ping:f64,
    client_bytes_sec:f32,
    server_bytes_sec:f32,
    input:Input,
    updates:u64,
    pub server_messages:Vec<ServerMsg>,
    pub client_messages:Vec<ClientMsg>
}

pub type KeyCode = u32;


#[derive(Clone)]
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


impl App {
    pub fn new() -> Self {
        Self {
            player_name:get_item("player_name").unwrap_or_default(),
            debug:true,
            app_state:AppState::Initial,
            canvas:Canvas::new(),
            state:State::new(),
            input:Input {
                pos:[0.0, 0.0].into(),
                dir:[0.0, 0.0].into(),
                ability_activated:false,
                thing_id:None,
                target_pos:Vec2::new(0.0, 0.0)
            },
            server_messages:Vec::new(),
            connection_status:"Not connected!".into(),
            client_messages:Vec::new(),
            id:Uuid::new_v4(),
            ping:0.0,
            server_bytes_sec:0.0,
            client_bytes_sec:0.0,
            updates:0
        }
    }

    pub fn init(&mut self) {
        self.canvas.set_image_src(0, "dummy.png");
    }

    pub fn draw(&self) {
        self.canvas.clear();
        let grid_size = 16.0;
        self.canvas.set_scale(grid_size);
 
        // draw things circle of things
        for (_, thing) in &self.state.things {
            let x = thing.pos.x as f64;
            let y = thing.pos.y as f64;
            self.canvas.draw_circle(x, y, thing.radius as f64);
        }
 
        // draw names of things
        for (_, thing) in &self.state.things {
            let x = thing.pos.x as f64;
            let y = thing.pos.y as f64;
            if thing.name.len() > 0 {
                self.canvas.fill_text(&thing.name, x, y - 1.0);
            }
        }

        let cx = (self.canvas.width() / grid_size as u32 / 2) as f64;
        let cy =  (self.canvas.height() / grid_size as u32 / 2) as f64;

        self.draw_ui_gameui();
        self.draw_ui_debug(grid_size);
        self.draw_ui_centercontent(cx, cy);
    }

    fn draw_ui_gameui(&self) {
        self.canvas.set_text_style("left", "middle");
        self.canvas.fill_text("100%", 0.0, 0.5);
    }

    fn draw_ui_centercontent(&self, cx: f64, cy: f64) {
        self.canvas.set_text_style("center", "middle");
        match &self.app_state {
            AppState::Initial | AppState::ReadyToJoin => {
                self.canvas.fill_text(&self.connection_status, cx, cy);
            },
            AppState::EnterName { name } => {
                self.canvas.fill_text(&format!("please enter your name and press enter"), cx, cy);
                self.canvas.fill_text(name, cx, cy + 1.0);
            },
            AppState::InGame {  } => {

            },
        };
    }

    fn draw_ui_debug(&self, grid_size: f64) {
        if self.debug {
            self.canvas.set_text_style("right", "middle");
            self.canvas.fill_text(format!("ping:{:0.00} ms", self.ping).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 0.5);
            self.canvas.fill_text(format!("send:{:0.00} kb/s", self.client_bytes_sec / 1000.0).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 1.5);
            self.canvas.fill_text(format!("recv:{:0.00} kb/s", self.server_bytes_sec / 1000.0).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 2.5);
        }
    }

    pub fn send(&mut self, msg:ClientMsg) {
        self.client_messages.push(msg)
    }

    pub fn recv_custom(&mut self, msg:CustomMsg) {
        match msg {
            CustomMsg::ServerSnapshotFull { state } => {
                self.state = state;
            },
            CustomMsg::ServerPlayerThing {
                thing_id
            } => {
                self.input.thing_id = thing_id;
            }
            _=> {
                
            },
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
                let ping:f64 = performance_now() - tick;
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

    pub fn update(&mut self) {
        for msg in &self.server_messages.clone() {
            self.recv(msg);
        }

        if self.updates % 10 == 0 {
            self.send(ClientMsg::Ping {
                tick:performance_now()
            });
        }

        let mouse_pos = self.canvas.get_mouse_pos();

        self.state.update(Some(&mut self.input));
        self.send_custom(CustomMsg::ClientInput {
            input:self.input.clone()
        });
      
        self.draw();
        self.updates += 1; 
    }

    pub fn keyup(&mut self, code:KeyCode, _key:&str) {
        match &self.app_state {
            AppState::InGame {  } => {
                let i = &mut self.input;
                if code == 87 && i.dir.y == -1.0 {
                    i.dir.y = 0.0;
                }
                if code == 83 && i.dir.y == 1.0 {
                    i.dir.y = 0.0;
                }
                if code == 65 && i.dir.x == -1.0 {
                    i.dir.x = 0.0;
                }
                if code == 68 && i.dir.x == 1.0 {
                    i.dir.x = 0.0;
                }

                if code == 32 {
                    i.ability_activated = false;
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
                    i.dir.y = -1.0;
                }
                if code == 83 {
                    i.dir.y = 1.0;
                }
        
                if code == 65 {
                    i.dir.x = -1.0;
                }
                if code == 68 {
                    i.dir.x = 1.0;
                }
        
                if code == 32 {
                    i.ability_activated = true;
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

    }

    pub fn mousedown(&mut self, button:u32, x:f32, y:f32) {
        if button == 0 {
            self.input.ability_activated = true;
        }

        info!("{}", self.input.ability_activated);
    }

    pub fn mouseup(&mut self, button:u32, x:f32, y:f32) {
        if button == 0 {
            self.input.ability_activated = false;
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

use hostess::{Bincoded, ClientMsg, ServerMsg, log::info, uuid::Uuid};
use crate::{Input, CustomMsg, State, performance_now};
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
    InGame {
        
    }
}


impl App {
    pub fn new() -> Self {
        Self {
            player_name:String::default(),
            debug:true,
            app_state:AppState::Initial,
            canvas:Canvas::new(),
            state:State::new(),
            input:Input {
                position:[0.0, 0.0].into(),
                dir:[0.0, 0.0].into(),
                shoot:false,
                thing_id:None
            },
            server_messages:Vec::new(),
            connection_status:"Not connected!".into(),
            client_messages:Vec::new(),
            id:Uuid::new_v4(),
            ping:0.0,
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
            self.canvas.fill_text(format!("ping:{:0.00}ms", self.ping).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 0.5);
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
            },
            ServerMsg::Pong {
                tick
            } => {
                let ping:f64 = performance_now() - tick;
                self.ping = ping;
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

        self.updates += 1; 

        if self.updates % 10 == 0 {
            self.send(ClientMsg::Ping {
                tick:performance_now()
            });
        }

        self.state.update(Some(&mut self.input));

        self.send_custom(CustomMsg::ClientInput {
            input:self.input.clone()
        });
      
        self.draw();
    }

    pub fn keyup(&mut self, code:KeyCode, key:&str) {
        match &self.app_state {
            AppState::EnterName { name } => {

            },
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
                    i.shoot = false;
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
                    self.state_transition(AppState::ReadyToJoin {});
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
                    i.shoot = true;
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


      /*  if code == 32 {
            self.client_messages.push(ClientMsg::CustomMsg {
                msg:GameClientMsg::ClientInput {
                    position:None,
                    shoot:true
                }.to_bincode()
            });
        }*/
        //info!("{}", code);
    }

    fn state_transition(&mut self, new_app_state:AppState) {
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
        self.state_transition(AppState::EnterName {
            name:self.player_name.clone()
        });
    }

    pub fn disconnected(&mut self) {
        self.connection_status = "Trying to reconnect...".into();
        self.state_transition(AppState::Initial);
    }
}

unsafe impl Send for App {
}
unsafe impl Sync for App {
}
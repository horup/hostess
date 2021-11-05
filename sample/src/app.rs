
use hostess::{ClientMsg, ServerMsg, uuid::Uuid, Bincoded};
use crate::{Input, CustomMsg, State, performance_now};
use super::Canvas;

pub struct App {
    app_state:AppState,
    id:Uuid,
    canvas:Canvas,
    state:State,
    status:String,
    ping:f64,
    input:Input,
    updates:u64,
    pub server_messages:Vec<ServerMsg>,
    pub client_messages:Vec<ClientMsg>
}

pub type KeyCode = u32;


/// enum holdning the client app state
enum AppState {
    Initial,
    EnterName {
        name:String
    },
    InGame {
        
    }
}


impl App {
    pub fn new() -> Self {
        Self {
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
            status:"Not connected!".into(),
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


      /*  self.canvas.set_text_style("center", "middle");
        self.canvas.fill_text(&self.status, (self.canvas.width() / 2 / grid_size as u32) as f64, 0.5);
        self.canvas.set_text_style("right", "middle");
        self.canvas.fill_text(format!("ping:{:0.00}ms", self.ping).as_str(), self.canvas.width() as f64 / grid_size - 0.1, 0.5);
        */
    }

    pub fn send(&mut self, msg:ClientMsg) {
        self.client_messages.push(msg)
    }

    pub fn recv(&mut self, msg:&ServerMsg) {
        match msg {
            ServerMsg::LobbyJoined {  } => {
                self.status = "Connected to Server".into();
               
            },
            ServerMsg::Hosts {hosts} => {
                if let Some(host) = hosts.first() {
                    self.status = format!("Joining host {}..", host.id);
                    let id = host.id;
                    self.send(ClientMsg::JoinHost {
                        host_id:id
                    });
                }
            },
            ServerMsg::HostJoined {host} => {
                self.status = format!("✓ Joined host {} ✓ ", host.id);
            },
            ServerMsg::Pong {
                tick
            } => {
                let ping:f64 = performance_now() - tick;
                self.ping = ping;
            },
            ServerMsg::Custom { msg } => {
                let msg = CustomMsg::from_bincode(msg).unwrap();
                match msg {
                    CustomMsg::ServerSnapshotFull { state } => {
                        self.state = state;
                    },
                    CustomMsg::ServerPlayerThing {
                        thing_id
                    } => {
                        self.input.thing_id = thing_id;
                    }
                    CustomMsg::ClientInput { input } => {
                        
                    },
                }
            }
            _ => {}
        }
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
        self.client_messages.push(ClientMsg::CustomMsg {
            msg:CustomMsg::ClientInput {
                input:self.input.clone()
            }.to_bincode()
        });
        self.draw();
    }

    pub fn keyup(&mut self, code:KeyCode) {
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
    }

    pub fn keydown(&mut self, code:KeyCode) {
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

    pub fn connected(&mut self) {
        self.status = format!("Sending Hello");
        self.client_messages.push(ClientMsg::Hello {
            client_id:self.id.clone()
        });
    }

    pub fn disconnected(&mut self) {
        self.status = "Trying to reconnect...".into();
    }
}

unsafe impl Send for App {
}
unsafe impl Sync for App {
}
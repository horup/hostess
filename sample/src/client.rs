
use std::time::Instant;

use hostess::{ClientMsg, ServerMsg, log::info, uuid::Uuid};
use crate::{GameState, performance_now};
use super::Canvas;

pub struct Client {
    id:Uuid,
    canvas:Canvas,
    state:GameState,
    status:String,
    pub server_messages:Vec<ServerMsg>,
    pub client_messages:Vec<ClientMsg>
}

pub type KeyCode = u32;

impl Client {
    pub fn new() -> Self {
        Self {
            canvas:Canvas::new(),
            state:GameState::new(),
            server_messages:Vec::new(),
            status:"Not connected!".into(),
            client_messages:Vec::new(),
            id:Uuid::new_v4()
        }
    }

    pub fn init(&mut self) {
        self.canvas.set_image_src(0, "dummy.png");
    }

    pub fn draw(&self) {
        self.canvas.clear();
        let grid_size = 16.0;
        self.canvas.set_scale(grid_size);

        // draw debug circle of things
        for (_, thing) in &self.state.things {
            let x = thing.pos.x as f64;
            let y = thing.pos.y as f64;
            self.canvas.draw_circle(x, y, thing.radius as f64);
        }

        // draw things
        for (_, thing) in &self.state.things {
            let x = thing.pos.x as f64;
            let y = thing.pos.y as f64;
            self.canvas.draw_normalized_image(0, x, y);
        }

        // draw names of things
        for (_, thing) in &self.state.things {
            let x = thing.pos.x as f64;
            let y = thing.pos.y as f64;
            self.canvas.fill_text(&thing.name, x, y - 1.0);
        }

        self.canvas.fill_text(&self.status, (self.canvas.width() / 2 / grid_size as u32) as f64, 0.5);
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
                info!("ping {} ms", performance_now() - tick)
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        for msg in &self.server_messages.clone() {
            self.recv(msg);
        }

        self.draw();
    }

    pub fn keyup(&mut self, _code:KeyCode) {
    }

    pub fn keydown(&mut self, _code:KeyCode) {
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

        self.send(ClientMsg::Ping {
            tick:performance_now()
        });

        info!("{}", _code);
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

unsafe impl Send for Client {
}
unsafe impl Sync for Client {
}
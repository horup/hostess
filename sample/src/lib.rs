#[allow(dead_code)]
mod app;
use app::*;

mod state;
pub use state::*;

mod thing;
pub use thing::*;

mod msg;
pub use msg::*;

mod input;
pub use input::*;

mod platform;
use hostess::{ServerMsg, log::{error, info}};
pub use platform::*;


use wasm_bindgen::prelude::*;

static mut APP:Option<App> = None;
static mut last_tick:f64 = 0.0;

#[wasm_bindgen]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());

    unsafe {
        let mut client = App::new();
        client.init();
        APP = Some(client);
    }
}

#[wasm_bindgen]
pub fn update() {
    unsafe {
        if let Some(client) = &mut APP {
            let mut dt = performance_now() - last_tick;
            dt /= 1000.0;
            if dt > 1.0 {
                dt = 1.0;
            }
            last_tick = performance_now();
            client.update(dt);
            client.server_messages.clear();

            for msg in &client.client_messages {
                match bincode::serialize(msg) {
                    Ok(v) => {
                        send(&v);
                    }
                    Err(v) => {
                        error!("{:?}", v);
                    }
                }
            }

            client.client_messages.clear();
        }
    }
}

#[wasm_bindgen]
pub fn keyup(keycode:u32, key:&str) {
    unsafe {
        if let Some(client) = &mut APP {
            client.keyup(keycode, key);
        }
    }
}

#[wasm_bindgen]
pub fn keydown(keycode:u32, key:&str) {
    unsafe {
        if let Some(client) = &mut APP {
            client.keydown(keycode, key);
        }
    }
}

#[wasm_bindgen]
pub fn mousedown(button:u32, x:f32, y:f32) {
    unsafe {
        if let Some(client) = &mut APP {
            client.mousedown(button, x, y);
        }
    }
}


#[wasm_bindgen]
pub fn mouseup(button:u32, x:f32, y:f32) {
    unsafe {
        if let Some(client) = &mut APP {
            client.mouseup(button, x, y);
        }
    }
}

#[wasm_bindgen]
pub fn mousemove(x:f32, y:f32) {
    unsafe {
        if let Some(client) = &mut APP {
            client.mousemove(x, y);
        }
    }
}

#[wasm_bindgen]
pub fn connected() {
    unsafe {
        if let Some(client) = &mut APP {
            client.client_messages.clear();
            client.server_messages.clear();
            client.connected();
        }
    }
}

#[wasm_bindgen]
pub fn disconnected() {
    unsafe {
        if let Some(client) = &mut APP {
            client.client_messages.clear();
            client.server_messages.clear();
            client.disconnected();
        }
    }
}

#[wasm_bindgen]
pub fn message(data:&[u8]) {
    unsafe {
        if let Some(client) = &mut APP {
            match bincode::deserialize::<ServerMsg>(data) {
                Ok(msg) => {
                    client.server_messages.push(msg);
                }
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub fn send(data:&[u8]);
}

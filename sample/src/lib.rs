#[allow(dead_code)]
mod app;
use app::*;

mod state;
pub use state::*;

mod msg;
pub use msg::*;

mod platform;
use hostess::{ServerMsg, log::{error}};
pub use platform::*;


use wasm_bindgen::prelude::*;

static mut GLOBAL_CLIENT:Option<App> = None;

#[wasm_bindgen]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());

    unsafe {
        let mut client = App::new();
        client.init();
        GLOBAL_CLIENT = Some(client);
    }
}

#[wasm_bindgen]
pub fn update() {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.update();
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
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.keyup(keycode, key);
        }
    }
}

#[wasm_bindgen]
pub fn keydown(keycode:u32, key:&str) {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.keydown(keycode, key);
        }
    }
}

#[wasm_bindgen]
pub fn connected() {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.client_messages.clear();
            client.server_messages.clear();
            client.connected();
        }
    }
}

#[wasm_bindgen]
pub fn disconnected() {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.client_messages.clear();
            client.server_messages.clear();
            client.disconnected();
        }
    }
}

#[wasm_bindgen]
pub fn message(data:&[u8]) {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
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

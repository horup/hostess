mod platform;
use hostess::{ServerMsg, log::{error, info}};
pub use platform::*;

mod client;
use client::*;

use wasm_bindgen::prelude::*;

static mut GLOBAL_CLIENT:Option<Client> = None;

#[wasm_bindgen]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());

    unsafe {
        let mut client = Client::new();
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
pub fn keyup(keycode:u32) {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.keyup(keycode);
        }
    }
}

#[wasm_bindgen]
pub fn keydown(keycode:u32) {
    unsafe {
        if let Some(client) = &mut GLOBAL_CLIENT {
            client.keydown(keycode);
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
                    info!("{:?}", msg);
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

#[cfg(not(target_arch = "wasm32"))]
pub use tokio;

#[cfg(not(target_arch = "wasm32"))]
pub mod host;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(not(target_arch = "wasm32"))]
pub mod lobby;

#[cfg(not(target_arch = "wasm32"))]
pub mod game_server;

#[cfg(not(target_arch = "wasm32"))]
pub mod untyped_game_server;

pub use log;
pub use uuid;
mod protocols;
pub use protocols::*;
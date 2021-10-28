#[cfg(not(target_arch = "wasm32"))]
mod host;

#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
pub use server::*;

#[cfg(not(target_arch = "wasm32"))]
mod lobby;

#[cfg(not(target_arch = "wasm32"))]
pub mod game_server;

mod untyped_game_server;
pub use untyped_game_server::*;

pub use log;

pub use uuid;

#[cfg(not(target_arch = "wasm32"))]
pub use tokio;

mod protocols;
pub use protocols::*;
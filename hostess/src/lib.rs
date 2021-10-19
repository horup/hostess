#[cfg(not(target_arch = "wasm32"))]
mod host;

#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
pub use server::*;

#[cfg(not(target_arch = "wasm32"))]
mod lobby;

#[cfg(not(target_arch = "wasm32"))]
pub mod typed_game;

mod game;
pub use game::*;

pub use log;

pub use uuid;

#[cfg(not(target_arch = "wasm32"))]
pub use tokio;

mod protocols;
pub use protocols::*;
pub use log;
pub use uuid;

#[cfg(not(target_arch = "wasm32"))]
pub mod master;

#[cfg(not(target_arch = "wasm32"))]
/**
all types needed to create a game server.
only valid for native targets, i.e. non-wasm32
*/
pub mod server;
/** 
all types needed for a client.
valid both for native targets and wasm32 targets.
*/
pub mod client;
pub mod bincoded;
pub mod shared;
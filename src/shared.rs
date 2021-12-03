use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HostInfo {
    pub id:Uuid,
    pub creator:Uuid,
    pub max_players:u32,
    pub current_players:u32
}
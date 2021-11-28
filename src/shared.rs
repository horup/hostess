use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostInfo {
    pub id:Uuid,
    pub creator:Uuid
}
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Hello {
        client_id:Uuid
    },
    CreateHost {

    },
    JoinHost {
        host_id:Uuid
    },
    LeaveHost {
    },
    RefreshHosts,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostInfo {
    pub id:Uuid,
    pub creator:Uuid
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    LobbyJoined {

    },
    HostCreated {
        host_id:Uuid
    },
    Hosts {
        hosts:Vec<HostInfo>
    },
    HostJoined {
        host:HostInfo
    }
}

impl ClientMsg {
    pub fn from_bincode(bytes:&[u8]) -> Option<Self> {
        let res = bincode::deserialize::<Self>(bytes);
        match res {
            Ok(msg) => return Some(msg),
            Err(_) => return None,
        }
    }
    
    pub fn to_bincode(&self) -> Vec<u8> {
        let res = bincode::serialize::<Self>(self);
        match res {
            Ok(bytes) => return bytes,
            Err(_) => return Vec::new(),
        }
    }
}

impl ServerMsg {
    pub fn from_bincode(bytes:&[u8]) -> Option<Self> {
        let res = bincode::deserialize::<Self>(bytes);
        match res {
            Ok(msg) => return Some(msg),
            Err(_) => return None,
        }
    }

    pub fn to_bincode(&self) -> Vec<u8> {
        let res = bincode::serialize::<Self>(self);
        match res {
            Ok(bytes) => return bytes,
            Err(_) => return Vec::new(),
        }
    }
}

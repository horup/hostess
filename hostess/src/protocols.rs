use uuid::Uuid;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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
    CustomMsg {
        msg:Vec<u8>
    },
    Ping {
        tick:f64
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
    },
    Pong {
        tick:f64
    },
    Custom {
        msg:Vec<u8>
    }
}

pub trait Bincoded : Sized + DeserializeOwned + 'static + Serialize {
    fn from_bincode(bytes:&[u8]) -> Option<Self> {
        let res = bincode::deserialize::<Self>(bytes);
        match res {
            Ok(msg) => return Some(msg),
            Err(_) => return None,
        }
    }

    fn to_bincode(&self) -> Vec<u8> {
        let res = bincode::serialize::<Self>(self);
        match res {
            Ok(bytes) => return bytes,
            Err(_) => return Vec::new(),
        }
    }
}

impl Bincoded for ClientMsg {
    
}

impl Bincoded for ServerMsg {

}
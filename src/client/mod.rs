pub use crate::shared::{HostInfo};
pub use uuid::Uuid;
pub use serde::{Deserialize, Serialize};
pub use crate::bincoded::Bincoded;


#[derive(Clone, Debug, Serialize, Deserialize)]
/// message sent from Client to Server
pub enum ClientMsg {
    Hello {
        client_id:Uuid,
        client_name:String
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
/// message sent from Server to Client
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
        tick:f64,

        /// number of bytes send from the server to the client per second
        /// on the application level only, i.e. does not account for websocket and tcp overhead
        server_bytes_sec:f32,

        /// number of bytes send from the client to the server per second
        /// on the application level only, i.e. does not account for websocket and tcp overhead
        client_bytes_sec:f32
    },
    Custom {
        msg:Vec<u8>
    }
}


impl Bincoded for ClientMsg {
}

impl Bincoded for ServerMsg {
}
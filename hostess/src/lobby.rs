use std::collections::HashMap;

use log::info;
use uuid::Uuid;
use crate::HostInfo;

pub struct Lobby {
    pub hosts:HashMap<Uuid, HostInfo>
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            hosts:HashMap::new()
        }
    }

    pub fn new_host(&mut self, creator:Uuid) -> Uuid {
        let host_id = Uuid::new_v4();
        let host = HostInfo {
            id:host_id,
            creator:creator
        };
        self.hosts.insert(host_id, host);
        info!("Host {:?} created by client {}", host_id, creator);
        return host_id;
    }

    pub fn hosts(&self) -> Vec<HostInfo> {
        let list = self.hosts.iter().map(|(_, host)| host.clone()).collect();
        return list;
    }
}
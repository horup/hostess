use std::{collections::HashMap};

use log::info;
use uuid::Uuid;
use crate::{HostInfo, host::Host, game_server::GameServerConstructor};

pub struct Lobby {
    hosts:HashMap<Uuid, Host>
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            hosts:HashMap::new()
        }
    }

    pub fn new_host(&mut self, creator:Uuid, constructor:GameServerConstructor) -> Uuid {
        let host_id = Uuid::new_v4();
        let host = Host::new(HostInfo {
            id:host_id,
            creator:creator
        }, constructor);

        self.hosts.insert(host_id, host);
        info!("Host {:?} created by client {}", host_id, creator);
        return host_id;
    }

    pub fn hosts(&self) -> Vec<HostInfo> {
        let list = self.hosts.iter().map(|(_, host)| host.info.clone()).collect();
        return list;
    }

    pub fn get_host(&self, id:Uuid) -> Option<Host> {
        if let Some(host) = self.hosts.get(&id) {
            return Some(host.clone());
        }

        None
    }
}
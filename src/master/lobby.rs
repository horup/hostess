use std::{collections::HashMap, sync::Arc};

use log::info;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::{server::{Server, Constructor}};

use super::host::Host;
use crate::shared::HostInfo;

pub struct Lobby {
    hosts:HashMap<Uuid, Host>
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            hosts:HashMap::new()
        }
    }

    pub fn new_host(&mut self, creator:Uuid, constructor:Constructor) -> Uuid {
        let host_id = Uuid::new_v4();
        let host = Host::new(Arc::new(RwLock::new(HostInfo {
            id:host_id,
            creator:creator,
            max_players:0,
            current_players:0
        })), constructor);

        self.hosts.insert(host_id, host);
        info!("Host {:?} created by client {}", host_id, creator);
        return host_id;
    }

    pub async fn hosts(&self) -> Vec<HostInfo> {
        let mut list = Vec::new();
        for (_, host) in self.hosts.iter() {
            list.push(host.info.read().await.clone());
        }
       
        return list;
    }

    pub fn get_host(&self, id:Uuid) -> Option<Host> {
        if let Some(host) = self.hosts.get(&id) {
            return Some(host.clone());
        }

        None
    }
}
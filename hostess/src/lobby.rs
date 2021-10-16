use std::{collections::HashMap, marker::PhantomData};

use log::info;
use uuid::Uuid;
use crate::{Game, HostInfo, host::Host};

pub struct Lobby<T:Game + Send> {
    hosts:HashMap<Uuid, Host>,
    phantom:PhantomData<T>
}

impl<T:Game + Send> Lobby<T> {
    pub fn new() -> Self {
        Lobby {
            hosts:HashMap::new(),
            phantom:PhantomData::default()
        }
    }

    pub fn new_host(&mut self, creator:Uuid) -> Uuid {
        let host_id = Uuid::new_v4();
        let host = Host::new::<T>(HostInfo {
            id:host_id,
            creator:creator
        });

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
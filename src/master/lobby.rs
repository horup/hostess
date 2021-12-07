use std::{collections::HashMap, sync::Arc};

use log::info;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::{server::{Server, Constructor}};

use super::instance::Instance;
use crate::shared::InstanceInfo;

pub struct Lobby {
    instances:HashMap<Uuid, Instance>
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            instances:HashMap::new()
        }
    }

    pub fn new_instance(&mut self, creator:Uuid, constructor:Constructor) -> Uuid {
        let id = Uuid::new_v4();
        let instance = Instance::new(Arc::new(RwLock::new(InstanceInfo {
            id,
            creator:creator,
            max_players:0,
            current_players:0
        })), constructor);

        self.instances.insert(id, instance);
        info!("Host {:?} created by client {}", id, creator);
        return id;
    }

    pub async fn instances(&self) -> Vec<InstanceInfo> {
        let mut list = Vec::new();
        for (_, host) in self.instances.iter() {
            list.push(host.info.read().await.clone());
        }
       
        return list;
    }

    pub fn get_instance(&self, id:Uuid) -> Option<Instance> {
        if let Some(host) = self.instances.get(&id) {
            return Some(host.clone());
        }

        None
    }
}
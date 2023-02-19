use std::collections::{HashMap, HashSet};

use btleplug::platform::Peripheral;
use uuid::Uuid;

use crate::clients::Clients;

pub struct ConnectedDevice {
    pub clients: HashSet<Uuid>,
    pub peripheral: Peripheral,
}

pub struct AppState {
    pub clients: Clients,
    pub discovery_clients: HashSet<Uuid>,
    pub connected_devices: HashMap<String, ConnectedDevice>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            clients: Clients::new(),
            discovery_clients: HashSet::new(),
            connected_devices: HashMap::new(),
        }
    }
}

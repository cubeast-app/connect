use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde()]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: Option<String>,
    pub address: Option<String>,
    pub signal_strength: Option<i16>,
    pub manufacturer_data: Option<HashMap<u16, Vec<u8>>>,
}

impl PartialEq for DiscoveredDevice {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
            && self.address.eq(&other.address)
            && self.manufacturer_data.eq(&other.manufacturer_data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum Broadcast {
    DiscoveredDevices {
        devices: Vec<DiscoveredDevice>,
    },
    CharacteristicValue {
        device: String,
        service: String,
        characteristic: String,
        value: String,
    },
    Disconnected,
}

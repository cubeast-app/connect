use std::collections::HashMap;

use btleplug::api::PeripheralProperties;
use serde::{Deserialize, Serialize};

type ManufacturerData = Option<HashMap<u16, Vec<u8>>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: Option<String>,
    pub address: Option<String>,
    pub signal_strength: Option<i16>,
    pub manufacturer_data: ManufacturerData,
}

impl From<(String, PeripheralProperties)> for DiscoveredDevice {
    fn from(properties: (String, PeripheralProperties)) -> Self {
        Self {
            id: properties.0,
            name: properties.1.local_name,
            signal_strength: properties.1.rssi,
            address: Some(properties.1.address.to_string()),
            manufacturer_data: Some(properties.1.manufacturer_data),
        }
    }
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
    DiscoveryStopped,
    CharacteristicValue {
        device: String,
        service: String,
        characteristic: String,
        value: String,
    },
    Disconnected,
}

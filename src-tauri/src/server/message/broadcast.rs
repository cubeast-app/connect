use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bluetooth::discovery::discovered_device::DiscoveredDevice;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Broadcast {
    DiscoveredDevices {
        devices: Vec<DiscoveredDevice>,
    },
    DiscoveryStopped,
    CharacteristicValue {
        device_name: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    },
    Disconnected {
        name: String,
    },
}

impl Display for Broadcast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoveredDevices { .. } => write!(f, "DiscoveredDevices"),
            Self::DiscoveryStopped => write!(f, "DiscoveryStopped"),
            Self::CharacteristicValue { .. } => write!(f, "CharacteristicValue"),
            Self::Disconnected { .. } => write!(f, "Disconnected"),
        }
    }
}

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{connected_device::DeviceId, discovered_device::DiscoveredDevice};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum Broadcast {
    DiscoveredDevices {
        devices: Vec<DiscoveredDevice>,
    },
    DiscoveryStopped,
    CharacteristicValue {
        device_id: DeviceId,
        characteristic_id: Uuid,
        value: Vec<u8>,
    },
    Disconnected {
        device_id: DeviceId,
    },
}

impl Display for Broadcast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Broadcast::DiscoveredDevices { .. } => write!(f, "DiscoveredDevices"),
            Broadcast::DiscoveryStopped => write!(f, "DiscoveryStopped"),
            Broadcast::CharacteristicValue { .. } => write!(f, "CharacteristicValue"),
            Broadcast::Disconnected { .. } => write!(f, "Disconnected"),
        }
    }
}

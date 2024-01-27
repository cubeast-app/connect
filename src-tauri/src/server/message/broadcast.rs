use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{bluetooth::discovery::discovered_device::DiscoveredDevice, device_id::DeviceId};

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
            Self::DiscoveredDevices { .. } => write!(f, "DiscoveredDevices"),
            Self::DiscoveryStopped => write!(f, "DiscoveryStopped"),
            Self::CharacteristicValue { .. } => write!(f, "CharacteristicValue"),
            Self::Disconnected { .. } => write!(f, "Disconnected"),
        }
    }
}

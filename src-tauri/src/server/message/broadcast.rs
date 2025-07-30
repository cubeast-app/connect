use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{bluetooth::discovery::discovered_device::DiscoveredDevice, app_status::Status};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Broadcast {
    DiscoveredDevices {
        devices: Vec<DiscoveredDevice>,
    },
    CharacteristicValue {
        timestamp: u64,
        device_id: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    },
    Disconnected {
        device_id: String,
    },
    StatusChanged {
        status: Status,
    },
}

impl Display for Broadcast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoveredDevices { .. } => write!(f, "DiscoveredDevices"),
            Self::CharacteristicValue { .. } => write!(f, "CharacteristicValue"),
            Self::Disconnected { .. } => write!(f, "Disconnected"),
            Self::StatusChanged { .. } => write!(f, "StatusChanged"),
        }
    }
}

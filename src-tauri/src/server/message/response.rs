use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::discovered_device::DiscoveredDevice;

#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Response {
    Ok,
    Error {
        error: String,
    },
    Value {
        value: Vec<u8>,
    },
    Version {
        version: u16,
    },
    Connected {
        device: DiscoveredDevice,
        services: Vec<Uuid>,
    },
}

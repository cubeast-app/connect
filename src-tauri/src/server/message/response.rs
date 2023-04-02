use serde::{Deserialize, Serialize};

use super::broadcast::DiscoveredDevice;

#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Response {
    Ok,
    Error {
        error: String,
    },

    Version {
        version: u16,
    },
    Connected {
        device: DiscoveredDevice,
        services: Vec<String>,
    },
}

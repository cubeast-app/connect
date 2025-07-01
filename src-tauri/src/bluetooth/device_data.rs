use serde::{Deserialize, Serialize};
use service::ServiceData;

use super::connected_device::ConnectedDevice;

pub mod characteristic;
pub mod service;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceData {
    pub id: String,
    pub name: Option<String>,
    pub services: Vec<ServiceData>,
}

impl From<&ConnectedDevice> for DeviceData {
    fn from(device: &ConnectedDevice) -> Self {
        let services = device.services.values().map(|c| c.into()).collect();

        Self {
            id: device.device.id.to_string(),
            name: device.device.name.clone(),
            services,
        }
    }
}

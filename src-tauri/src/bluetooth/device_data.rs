use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use service::ServiceData;

use super::connected_device::ConnectedDevice;

pub mod characteristic;
pub mod service;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceData {
    pub id: String,
    pub name: Option<String>,
    pub address: Option<String>,
    pub manufacturer_data: Option<HashMap<u16, Vec<u8>>>,
    pub services: Vec<ServiceData>,
}

impl From<&ConnectedDevice> for DeviceData {
    fn from(device: &ConnectedDevice) -> Self {
        let services = device.services.values().map(|c| c.into()).collect();

        Self {
            id: device.device.id.to_string(),
            name: device.device.name.clone(),
            address: device.device.address.clone(),
            manufacturer_data: device.device.manufacturer_data.clone(),
            services,
        }
    }
}

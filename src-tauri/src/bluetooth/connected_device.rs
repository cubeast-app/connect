use std::collections::HashMap;

use btleplug::{
    api::{Peripheral, Service},
    platform::Peripheral as PlatformPeripheral,
    Error,
};

use super::{discovery::discovered_device::DiscoveredDevice, notifications::Notifications};

pub struct ConnectedDevice {
    pub peripheral: PlatformPeripheral,
    pub device: DiscoveredDevice,
    pub services: HashMap<String, Service>,
    pub client_count: usize,
    pub notifications: Notifications,
}

impl ConnectedDevice {
    pub async fn start(peripheral: PlatformPeripheral, device_name: String) -> Result<Self, Error> {
        let properties = peripheral
            .properties()
            .await?
            .ok_or(Error::DeviceNotFound)?;
        let discovered_device: DiscoveredDevice = (device_name, properties).into();
        peripheral.discover_services().await?;
        let services = peripheral.services();

        // services BTree as HashMap
        let services: HashMap<_, _> = services
            .into_iter()
            .map(|s| (s.uuid.to_string(), s))
            .collect();

        let notifications = Notifications::start(peripheral.clone());
        Ok(Self::new(
            peripheral,
            discovered_device,
            services,
            notifications,
        ))
    }

    fn new(
        peripheral: PlatformPeripheral,
        device: DiscoveredDevice,
        services: HashMap<String, Service>,
        notifications: Notifications,
    ) -> Self {
        Self {
            peripheral,
            device,
            services,
            client_count: 0,
            notifications,
        }
    }

    pub fn add_client(&mut self) {
        self.client_count += 1;
    }

    pub fn remove_client(&mut self) {
        if self.client_count > 0 {
            self.client_count -= 1;
        }
    }

    pub fn has_no_clients(&self) -> bool {
        self.client_count == 0
    }
}

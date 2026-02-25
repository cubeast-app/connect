use std::collections::HashMap;

use btleplug::{
    api::{Peripheral, Service},
    platform::Peripheral as PlatformPeripheral,
    Error,
};
use tracing::{info, warn};

use super::{discovery::discovered_device::DiscoveredDevice, notifications::Notifications};

const DISCOVER_RETRIES: u32 = 3;
const DISCOVER_RETRY_DELAY_MS: u64 = 1_000;

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

        let mut last_err = Error::DeviceNotFound;
        for attempt in 1..=DISCOVER_RETRIES {
            match peripheral.discover_services().await {
                Ok(()) => {
                    if attempt > 1 {
                        info!("Service discovery succeeded on attempt {attempt}");
                    }
                    last_err = Error::DeviceNotFound; // won't be used
                    break;
                }
                Err(e) => {
                    warn!("Service discovery attempt {attempt}/{DISCOVER_RETRIES} failed: {e:?}");
                    last_err = e;
                    if attempt < DISCOVER_RETRIES {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            DISCOVER_RETRY_DELAY_MS,
                        ))
                        .await;
                    }
                }
            }
        }
        // Re-check by trying to read services; if the loop exhausted all retries the
        // last error is propagated.
        if peripheral.services().is_empty() {
            return Err(last_err);
        }

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

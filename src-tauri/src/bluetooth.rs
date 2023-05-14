use std::collections::HashMap;

use btleplug::{
    api::{Central, Manager as _, Peripheral},
    platform::Adapter,
    platform::Manager,
    Error,
};
use futures_util::stream::AbortHandle;
use log::{info, warn};
use tokio::sync::mpsc::{error::SendError, Sender};
use uuid::Uuid;

use crate::{
    connected_device::{ConnectedDevice, DeviceId},
    disconnections::Disconnections,
    discovered_device::DiscoveredDevice,
    discovery::discovery::Discovery,
    events::Events,
    notifications::notifications::Notifications,
};

pub struct Bluetooth {
    pub events: Option<Events>,
    discovery_abort: Option<Sender<()>>,
    notification_aborts: HashMap<DeviceId, AbortHandle>,
}

impl Bluetooth {
    pub fn new() -> Self {
        Self {
            events: None,
            discovery_abort: None,
            notification_aborts: HashMap::new(),
        }
    }

    pub async fn start(&self) -> Result<(), Error> {
        info!("Starting Bluetooth");

        Disconnections::start(
            Self::adapter().await?,
            self.events.clone().expect("Events not set"),
        );

        Ok(())
    }

    async fn adapter() -> Result<Adapter, Error> {
        let manager = Manager::new().await?;

        let adapters = manager.adapters().await?;
        adapters
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotSupported("No Bluetooth adapters found".to_string()))
    }

    pub async fn start_discovery(&mut self) -> Result<(), Error> {
        if self.discovery_abort.is_some() {
            return Ok(());
        }

        let adapter = Self::adapter().await?;

        if let Some(events) = &self.events {
            self.discovery_abort = Some(Discovery::start(adapter, events.clone()).await);
        }

        Ok(())
    }

    pub async fn stop_discovery(&mut self) -> Result<(), SendError<()>> {
        if let Some(abort) = &self.discovery_abort {
            abort.send(()).await?;
            self.discovery_abort = None;
        }

        Ok(())
    }

    pub async fn connect_device(&mut self, device_id: String) -> Result<ConnectedDevice, Error> {
        let adapter = Self::adapter().await?;
        let peripherals = adapter.peripherals().await?;
        let peripheral = peripherals.iter().find(|p| p.id().to_string() == device_id);

        if let Some(peripheral) = peripheral {
            if let Some(events) = &self.events {
                peripheral.connect().await?;
                let properties = peripheral
                    .properties()
                    .await?
                    .ok_or(Error::DeviceNotFound)?;
                let discovered_device: DiscoveredDevice = (device_id.clone(), properties).into();
                peripheral.discover_services().await?;

                // services BTree as Vec
                let services: Vec<Uuid> = peripheral.services().iter().map(|s| s.uuid).collect();
                let notification_stream = peripheral.notifications().await?;

                let notification_abort =
                    Notifications::start(device_id.clone(), notification_stream, events.clone());

                self.notification_aborts
                    .insert(device_id.clone(), notification_abort);

                info!("Connected to {}", device_id);

                let connected_device =
                    ConnectedDevice::new(peripheral.clone(), discovered_device, services);

                Ok(connected_device)
            } else {
                Err(Error::NotConnected)
            }
        } else {
            Err(Error::DeviceNotFound)
        }
    }

    pub async fn disconnect_device(
        &mut self,
        connected_device: &ConnectedDevice,
    ) -> Result<(), Error> {
        self.clean_up_device(&connected_device.device.id);

        let result = connected_device.peripheral.disconnect().await;

        if let Err(err) = result {
            info!(
                "Error disconnecting from {}: {}",
                connected_device.device.id, err
            );
        } else {
            info!("Disconnected from {}", connected_device.device.id);
        }

        Ok(())
    }

    pub fn clean_up_device(&mut self, device_id: &DeviceId) {
        let abort = self.notification_aborts.remove(device_id);

        if let Some(abort) = abort {
            abort.abort();
        } else {
            warn!("No notification abort found for {}", device_id);
        }
    }

    pub async fn disconnect_devices(
        &mut self,
        connected_devices: Vec<&ConnectedDevice>,
    ) -> Result<(), Error> {
        for connected_device in connected_devices {
            self.disconnect_device(connected_device).await?;
        }

        Ok(())
    }

    pub fn set_events(&mut self, events: Events) {
        self.events = Some(events)
    }
}

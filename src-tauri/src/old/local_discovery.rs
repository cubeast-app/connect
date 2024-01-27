use std::{future, pin::Pin};

use futures_util::Future;
use tokio::sync::mpsc::error::SendError;

use crate::{
    broadcaster::broadcast_command::BroadcastCommand, client::Client,
    discovered_device::DiscoveredDevice,
};

pub struct LocalDiscovery {
    discovered_devices: Vec<DiscoveredDevice>,
}

impl LocalDiscovery {
    pub fn new() -> Self {
        Self {
            discovered_devices: vec![],
        }
    }

    pub fn discovered_devices(&self) -> &[DiscoveredDevice] {
        &self.discovered_devices
    }
}

impl Client for LocalDiscovery {
    fn on_discovery(
        &mut self,
        discovered_devices: &[DiscoveredDevice],
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>> {
        self.discovered_devices = discovered_devices.to_vec();

        Box::pin(future::ready(Ok(())))
    }

    fn on_notification(
        &mut self,
        device_id: &crate::connected_device::DeviceId,
        characteristic_id: &uuid::Uuid,
        value: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>> {
        Box::pin(future::ready(Ok(())))
    }

    fn on_device_disconnected(
        &mut self,
        device_id: &crate::connected_device::DeviceId,
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>> {
        Box::pin(future::ready(Ok(())))
    }
}

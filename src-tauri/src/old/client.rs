use std::pin::Pin;

use futures_util::Future;
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;

use crate::{
    broadcaster::broadcast_command::BroadcastCommand, connected_device::DeviceId,
    discovered_device::DiscoveredDevice,
};

pub trait Client: Send + Sync {
    fn on_discovery(
        &mut self,
        discovered_devices: &[DiscoveredDevice],
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>>;
    fn on_notification(
        &mut self,
        device_id: &DeviceId,
        characteristic_id: &Uuid,
        value: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>>;
    fn on_device_disconnected(
        &mut self,
        device_id: &DeviceId,
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>>;
}

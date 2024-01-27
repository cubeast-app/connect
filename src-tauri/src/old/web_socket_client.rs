use std::pin::Pin;

use crate::{
    broadcaster::broadcast_command::BroadcastCommand, client::Client, clients::WebSocket,
    discovered_device::DiscoveredDevice, server::message::broadcast::Broadcast,
};
use futures_util::Future;
use tokio::sync::mpsc::{error::SendError, Sender};

pub struct WebSocketClient {
    websocket: WebSocket,
    broadcaster_tx: Sender<BroadcastCommand>,
}

impl WebSocketClient {
    pub fn new(websocket: WebSocket, broadcaster_tx: Sender<BroadcastCommand>) -> Self {
        Self {
            websocket,
            broadcaster_tx,
        }
    }
}

impl Client for WebSocketClient {
    fn on_discovery(
        &mut self,
        discovered_devices: &[DiscoveredDevice],
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>> {
        let broadcast = Broadcast::DiscoveredDevices {
            devices: discovered_devices.to_vec(),
        };

        let broadcast_command = BroadcastCommand {
            clients: vec![self.websocket.clone()],
            broadcast,
        };

        let future = self.broadcaster_tx.send(broadcast_command);
        Box::pin(future)
    }

    fn on_notification(
        &mut self,
        device_id: &crate::connected_device::DeviceId,
        characteristic_id: &uuid::Uuid,
        value: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>> {
        let broadcast = Broadcast::CharacteristicValue {
            device_id: device_id.clone(),
            characteristic_id: *characteristic_id,
            value: value.to_vec(),
        };

        let broadcast_command = BroadcastCommand {
            broadcast,
            clients: vec![self.websocket.clone()],
        };

        let future = self.broadcaster_tx.send(broadcast_command);
        Box::pin(future)
    }

    fn on_device_disconnected(
        &mut self,
        device_id: &crate::connected_device::DeviceId,
    ) -> Pin<Box<dyn Future<Output = Result<(), SendError<BroadcastCommand>>> + Send + '_>> {
        let broadcast = Broadcast::Disconnected {
            device_id: device_id.clone(),
        };
        let broadcast_command = BroadcastCommand {
            clients: vec![self.websocket.clone()],
            broadcast,
        };
        let future = self.broadcaster_tx.send(broadcast_command);
        Box::pin(future)
    }
}

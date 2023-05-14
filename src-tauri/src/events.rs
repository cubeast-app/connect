use std::sync::Arc;

use tauri::async_runtime::Mutex;
use tokio::sync::mpsc::{error::SendError, Sender};
use uuid::Uuid;

use crate::{
    bluetooth_state::BluetoothState, broadcaster::broadcast_command::BroadcastCommand,
    connected_device::DeviceId, discovered_device::DiscoveredDevice,
    notifications::notifications::CharacteristicValue, server::message::broadcast::Broadcast,
};

#[derive(Clone)]
pub struct Events {
    bluetooth_state: Arc<Mutex<BluetoothState>>,
    broadcaster_tx: Sender<BroadcastCommand>,
}

impl Events {
    pub fn new(
        bluetooth_state: Arc<Mutex<BluetoothState>>,
        broadcaster_tx: Sender<BroadcastCommand>,
    ) -> Self {
        Self {
            bluetooth_state,
            broadcaster_tx,
        }
    }

    pub async fn on_discovery(&self, discovered_devices: Vec<DiscoveredDevice>) {
        let bluetooth_state = self.bluetooth_state.lock().await;
        let clients = bluetooth_state.discovery_clients().await;
        let broadcast = Broadcast::DiscoveredDevices {
            devices: discovered_devices,
        };
        let broadcast_command = BroadcastCommand { clients, broadcast };

        self.broadcaster_tx
            .send(broadcast_command)
            .await
            .expect("Discovery broadcaster failed");
    }

    pub async fn on_notification(
        &self,
        device_id: String,
        characteristic_id: Uuid,
        value: CharacteristicValue,
    ) -> Result<(), SendError<BroadcastCommand>> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        let connected_device = bluetooth_state.connected_device(&device_id);

        if let Some(connected_device) = connected_device {
            let characteristic_subscribers = connected_device.subscriptions(&characteristic_id);

            if let Some(characteristic_subscribers) = characteristic_subscribers {
                // Clone client_ids to avoid the mutable borrow of bluetooth_state
                let client_ids = characteristic_subscribers
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();

                let clients = client_ids
                    .iter()
                    .filter_map(|client_id| bluetooth_state.client(client_id))
                    .collect::<Vec<_>>();

                self.broadcaster_tx
                    .send(BroadcastCommand {
                        broadcast: Broadcast::CharacteristicValue {
                            device_id,
                            characteristic_id,
                            value,
                        },
                        clients,
                    })
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn on_device_disconnected(
        &self,
        device_id: DeviceId,
    ) -> Result<(), SendError<BroadcastCommand>> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        let device = bluetooth_state.connected_device(&device_id);

        if let Some(device) = device {
            let clients_ids = device.client_ids();
            let clients = clients_ids
                .iter()
                .filter_map(|client_id| bluetooth_state.client(client_id))
                .collect::<Vec<_>>();

            self.broadcaster_tx
                .send(BroadcastCommand {
                    broadcast: Broadcast::Disconnected {
                        device_id: device_id.clone(),
                    },
                    clients,
                })
                .await?;

            bluetooth_state.device_disconnected(&device_id);
        }

        Ok(())
    }
}

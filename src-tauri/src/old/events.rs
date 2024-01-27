use std::sync::Arc;

use tauri::async_runtime::Mutex;
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;

use crate::{
    bluetooth_state::BluetoothState, broadcaster::broadcast_command::BroadcastCommand,
    connected_device::DeviceId, discovered_device::DiscoveredDevice,
    notifications::notifications::CharacteristicValue,
};

/*

#[derive(Clone)]
pub struct Events {
    bluetooth_state: Arc<Mutex<BluetoothState>>,
}

impl Events {
    pub fn new(bluetooth_state: Arc<Mutex<BluetoothState>>) -> Self {
        Self { bluetooth_state }
    }

    pub async fn on_discovery(&self, discovered_devices: Vec<DiscoveredDevice>) {
        /*
        let bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state.on_discovery(&discovered_devices).await;
        */
        todo!()
    }

    pub async fn on_notification(
        &self,
        device_id: String,
        characteristic_id: Uuid,
        value: CharacteristicValue,
    ) -> Result<(), SendError<BroadcastCommand>> {
        /*
        let bluetooth_state = self.bluetooth_state.lock().await;
        let connected_device = bluetooth_state.connected_device(&device_id);

        if let Some(connected_device) = connected_device {
            let characteristic_subscribers = connected_device.subscriptions(&characteristic_id);

            if let Some(characteristic_subscribers) = characteristic_subscribers {
                // Clone client_ids to avoid the mutable borrow of bluetooth_state
                let client_ids = characteristic_subscribers
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();

                let mut clients = client_ids
                    .iter()
                    .filter_map(|client_id| bluetooth_state.client(client_id))
                    .collect::<Vec<_>>();

                for client in clients.iter_mut() {
                    client
                        .on_notification(&device_id, &characteristic_id, &value)
                        .await;
                }
            }
        }

        Ok(())
        */
        todo!()
    }

    pub async fn on_device_disconnected(
        &self,
        device_id: DeviceId,
    ) -> Result<(), SendError<BroadcastCommand>> {
        /*
        let bluetooth_state = self.bluetooth_state.lock().await;
        let device = bluetooth_state.connected_device(&device_id);

        if let Some(device) = device {
            let clients_ids = device.client_ids();
            let mut clients = clients_ids
                .iter()
                .filter_map(|client_id| bluetooth_state.client(client_id))
                .collect::<Vec<_>>();

            for client in clients.iter_mut() {
                client.on_device_disconnected(&device_id);
            }

            bluetooth_state.device_disconnected(&device_id);
        }

        Ok(())
        */
        todo!()
    }
}

*/

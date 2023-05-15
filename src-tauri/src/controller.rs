use std::{sync::Arc, time::Duration};

use btleplug::Error;
use tauri::async_runtime::Mutex;
use tokio::time;
use uuid::Uuid;

use crate::{
    bluetooth_state::BluetoothState,
    clients::Client,
    connected_device::{ClientId, DeviceId},
    discovered_device::DiscoveredDevice,
};

const STOP_DISCOVERY_TIMEOUT: u64 = 60000;

#[derive(Clone)]
pub struct Controller {
    bluetooth_state: Arc<Mutex<BluetoothState>>,
}

impl Controller {
    pub fn new(bluetooth_state: Arc<Mutex<BluetoothState>>) -> Self {
        Self { bluetooth_state }
    }

    pub async fn add_client(&self, client: Client) -> Uuid {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state.client_connected(client)
    }

    pub async fn remove_client(&self, client_id: &Uuid) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state.client_disconnected(client_id).await?;

        Ok(())
    }

    pub async fn client_count(&self) -> usize {
        let bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state.client_count()
    }

    pub async fn add_discovery_client(&self, client_id: Uuid) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state.client_started_discovery(client_id).await?;

        let controller = self.clone();
        tokio::spawn(async move {
            time::sleep(Duration::from_millis(STOP_DISCOVERY_TIMEOUT)).await;

            controller
                .remove_discovery_client(&client_id)
                .await
                .expect("Failed to remove discovery client");
        });

        Ok(())
    }

    pub async fn remove_discovery_client(&self, client_id: &Uuid) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state
            .client_stopped_discovery(client_id)
            .await
            .map_err(|error| Error::Other(Box::new(error)))?;

        Ok(())
    }

    pub async fn connect_client_to_device(
        &self,
        device_id: DeviceId,
        client_id: Uuid,
    ) -> Result<(DiscoveredDevice, Vec<Uuid>), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state
            .connect_client_to_device(device_id, client_id)
            .await
    }

    pub async fn disconnect_client_from_device(
        &self,
        device_id: String,
        client_id: Uuid,
    ) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;

        let device_disconnected = bluetooth_state
            .disconnect_client_from_device(device_id.clone(), &client_id)
            .await?;

        if device_disconnected {
            Ok(())
        } else {
            Err(Error::NotConnected)
        }
    }

    pub async fn read_characteristic(
        &self,
        device_id: String,
        characteristic_id: Uuid,
    ) -> Result<Vec<u8>, Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state
            .read_characteristic(device_id, characteristic_id)
            .await
    }

    pub async fn write_characteristic(
        &self,
        device_id: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state
            .write_characteristic(device_id, characteristic_id, value)
            .await
    }

    pub async fn subscribe_characteristic(
        &self,
        client_id: ClientId,
        device_id: DeviceId,
        characteristic_id: Uuid,
    ) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state
            .subscribe_characteristic(client_id, device_id, characteristic_id)
            .await
    }

    pub async fn unsubscribe_characteristic(
        &self,
        device_id: String,
        characteristic_id: Uuid,
        client_id: &ClientId,
    ) -> Result<(), Error> {
        let mut bluetooth_state = self.bluetooth_state.lock().await;
        bluetooth_state
            .unsubscribe_characteristic(device_id, characteristic_id, client_id)
            .await
    }
}

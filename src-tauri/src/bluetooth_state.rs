use btleplug::api::Peripheral;
use btleplug::platform::Peripheral as PlatformPeripheral;
use btleplug::{api::Characteristic, Error};
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;

use crate::connected_device::{ClientId, ConnectedDevice, DeviceId};
use crate::events::Events;
use crate::{
    app_state::AppState, bluetooth::Bluetooth, clients::Client, discovered_device::DiscoveredDevice,
};

/// Maintains a combined state of the bluetooth adapter and the application data relevant to Bluetooth.
///
/// Those two states need to be kept in sync.
pub struct BluetoothState {
    state: AppState<Client>,
    bluetooth: Bluetooth,
}

impl BluetoothState {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            bluetooth: Bluetooth::new(),
        }
    }

    pub async fn start(&self) -> Result<(), Error> {
        self.bluetooth.start().await?;

        Ok(())
    }

    pub fn set_events(&mut self, events: Events) {
        self.bluetooth.set_events(events);
    }

    pub fn client_connected(&mut self, client: Client) -> Uuid {
        self.state.add_client(client)
    }

    pub async fn client_disconnected(&mut self, client_id: &Uuid) -> Result<(), Error> {
        self.state.remove_client(client_id);

        // stop discovery if needed
        if !self.state.has_discovery_clients() {
            self.bluetooth
                .stop_discovery()
                .await
                .map_err(|err| Error::Other(Box::new(err)))?;
        }

        // stop notifications if needed

        // disconnect devices with no clients
        self.bluetooth
            .disconnect_devices(self.state.connected_devices_with_no_clients())
            .await?;

        Ok(())
    }

    pub fn client_count(&self) -> usize {
        self.state.client_count()
    }

    pub fn client(&self, client_id: &Uuid) -> Option<Client> {
        self.state.client(client_id)
    }

    pub async fn client_started_discovery(&mut self, client_id: Uuid) -> Result<(), Error> {
        self.state.add_discovery_client(client_id);

        self.bluetooth.start_discovery().await?;

        Ok(())
    }

    pub async fn client_stopped_discovery(
        &mut self,
        client_id: &Uuid,
    ) -> Result<(), SendError<()>> {
        self.state.remove_discovery_client(client_id);

        if !self.state.has_discovery_clients() {
            self.bluetooth.stop_discovery().await?;
        }

        Ok(())
    }

    pub async fn discovery_clients(&self) -> Vec<Client> {
        self.state.discovery_clients()
    }

    pub async fn connect_client_to_device(
        &mut self,
        device_id: DeviceId,
        client_id: Uuid,
    ) -> Result<(DiscoveredDevice, Vec<Uuid>), Error> {
        self.connect_device_if_needed(device_id.clone()).await?;
        self.add_client_to_device(&device_id, client_id)
    }

    async fn connect_device_if_needed(&mut self, device_id: DeviceId) -> Result<(), Error> {
        if self.state.connected_device(&device_id).is_none() {
            let connected_device = self.bluetooth.connect_device(device_id).await?;
            self.state.add_connected_device(connected_device);
        }

        Ok(())
    }

    fn add_client_to_device(
        &mut self,
        device_id: &DeviceId,
        client_id: Uuid,
    ) -> Result<(DiscoveredDevice, Vec<Uuid>), Error> {
        let connected_device = self
            .state
            .connected_device(device_id)
            .ok_or_else(|| Error::DeviceNotFound)?;
        connected_device.add_client(client_id);

        Ok((
            connected_device.device.clone(),
            connected_device.services.clone(),
        ))
    }

    pub async fn disconnect_client_from_device(
        &mut self,
        device_id: String,
        client_id: &Uuid,
    ) -> Result<bool, Error> {
        self.remove_client_from_device(&device_id, client_id)?;

        let connected_device = self
            .state
            .connected_device(&device_id)
            .ok_or_else(|| Error::DeviceNotFound)?;

        if connected_device.has_no_clients() {
            self.bluetooth.disconnect_device(connected_device).await?;
            self.state.device_disconnected(&device_id);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn remove_client_from_device(
        &mut self,
        device_id: &String,
        client_id: &Uuid,
    ) -> Result<(), Error> {
        let connected_device = self
            .state
            .connected_device(device_id)
            .ok_or_else(|| Error::DeviceNotFound)?;
        connected_device.remove_client(client_id);
        Ok(())
    }

    pub fn connected_device(&mut self, device_id: &DeviceId) -> Option<&mut ConnectedDevice> {
        self.state.connected_device(device_id)
    }

    pub fn device_disconnected(&mut self, device_id: &DeviceId) {
        self.bluetooth.clean_up_device(device_id);
        self.state.device_disconnected(device_id)
    }

    pub async fn read_characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
    ) -> Result<Vec<u8>, Error> {
        let (peripheral, characteristic) = &self.characteristic(device_id, characteristic_id)?;
        peripheral.read(characteristic).await
    }

    pub fn characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
    ) -> Result<(PlatformPeripheral, Characteristic), Error> {
        let peripheral = &self
            .state
            .connected_device(&device_id)
            .ok_or(Error::DeviceNotFound)?
            .peripheral;
        let characteristic = peripheral
            .characteristics()
            .iter()
            .find(|c| c.uuid == characteristic_id)
            .ok_or(Error::NotSupported(format!(
                "Characteristic not found: {}",
                characteristic_id
            )))?
            .clone();

        Ok((peripheral.clone(), characteristic))
    }

    pub async fn write_characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        let peripheral = &self
            .state
            .connected_device(&device_id)
            .ok_or(Error::DeviceNotFound)?
            .peripheral;
        let characteristic = peripheral
            .characteristics()
            .iter()
            .find(|c| c.uuid == characteristic_id)
            .ok_or(Error::NotSupported(format!(
                "Characteristic not found: {}",
                characteristic_id
            )))?
            .clone();
        peripheral
            .write(
                &characteristic,
                &value,
                btleplug::api::WriteType::WithoutResponse,
            )
            .await
    }

    pub async fn subscribe_characteristic(
        &mut self,
        client_id: ClientId,
        device_id: DeviceId,
        characteristic_id: Uuid,
    ) -> Result<(), Error> {
        let connected_device = self
            .state
            .connected_device(&device_id)
            .ok_or(Error::DeviceNotFound)?;
        let characteristic_subscribers = connected_device.subscriptions(&characteristic_id);

        if let Some(characteristic_subscribers) = characteristic_subscribers {
            characteristic_subscribers.insert(client_id);
        } else {
            let peripheral = &connected_device.peripheral;
            let characteristic = peripheral
                .characteristics()
                .iter()
                .find(|c| c.uuid == characteristic_id)
                .ok_or(Error::NotSupported(format!(
                    "Characteristic not found: {}",
                    characteristic_id
                )))?
                .clone();
            peripheral.subscribe(&characteristic).await?;
            connected_device.add_subscription(client_id, characteristic_id);
        }

        Ok(())
    }

    pub async fn unsubscribe_characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
        client_id: &ClientId,
    ) -> Result<(), Error> {
        let connected_device = self
            .state
            .connected_device(&device_id)
            .ok_or(Error::DeviceNotFound)?;
        let peripheral = connected_device.peripheral.clone();
        let characteristic = peripheral
            .characteristics()
            .iter()
            .find(|c| c.uuid == characteristic_id)
            .ok_or(Error::NotSupported(format!(
                "Characteristic not found: {}",
                characteristic_id
            )))?
            .clone();

        let characteristic_subscribers = connected_device.subscriptions(&characteristic_id);
        if let Some(characteristic_subscribers) = characteristic_subscribers {
            characteristic_subscribers.remove(client_id);

            if characteristic_subscribers.is_empty() {
                peripheral.unsubscribe(&characteristic).await?;
                connected_device.remove_subscriptions_for_characteristic(&characteristic.uuid);
            }
        }

        Ok(())
    }
}

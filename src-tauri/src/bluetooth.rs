use std::collections::HashMap;

use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral},
    platform::{Adapter, Manager, Peripheral as PlatformPeripheral},
    Error,
};
use connected_device::ConnectedDevice;
use device_data::DeviceData;
use discovery::discovery_stream::DiscoveryStream;
use log::{error, info};
use notifications::notification_stream::NotificationStream;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::{self},
};
use uuid::Uuid;

use self::discovery::Discovery;

pub mod connected_device;
pub mod device_data;
pub mod discovery;
pub mod notifications;

enum BluetoothMessage {
    SubscribeToDiscovery(oneshot::Sender<Result<DiscoveryStream, Error>>),
    UnsubscribeFromDiscovery,
    Connect(String, oneshot::Sender<Result<DeviceData, Error>>),
    Disconnect(String, oneshot::Sender<Result<(), Error>>),
    ReadCharacteristic(String, Uuid, oneshot::Sender<Result<Vec<u8>, Error>>),
    WriteCharacteristic(String, Uuid, Vec<u8>, oneshot::Sender<Result<(), Error>>),
    SubscribeToCharacteristic(
        String,
        Uuid,
        oneshot::Sender<Result<NotificationStream, Error>>,
    ),
    UnsubscribeFromCharacteristic(String, Uuid, oneshot::Sender<Result<(), Error>>),
}

pub(crate) async fn adapter() -> Result<Adapter, Error> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    adapters
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotSupported("No Bluetooth adapters found".to_string()))
}

#[derive(Clone)]
pub(crate) struct Bluetooth {
    tx: UnboundedSender<BluetoothMessage>,
}

impl Bluetooth {
    pub fn start(adapter: Adapter) -> Self {
        let (tx, rx) = unbounded_channel();

        let discovery = Discovery::start(adapter.clone());
        let mut actor = BluetoothActor::new(adapter, discovery);

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self { tx }
    }

    pub async fn subscribe_to_discovery(&self) -> Result<DiscoveryStream, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::SubscribeToDiscovery(tx))
            .expect("Failed to send message to Bluetooth actor");

        rx.await.expect("Failed to receive discovery response")
    }

    pub async fn unsubscribe_from_discovery(&self) {
        self.tx
            .send(BluetoothMessage::UnsubscribeFromDiscovery)
            .expect("Failed to send message to Bluetooth actor");
    }

    pub async fn connect(&self, name: String) -> Result<DeviceData, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::Connect(name, tx))
            .expect("Failed to send message to Bluetooth actor");

        rx.await.expect("Failed to receive connect response")
    }

    pub async fn disconnect(&self, name: String) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::Disconnect(name, tx))
            .expect("Failed to send message to Bluetooth actor");

        rx.await.expect("Failed to receive disconnect response")
    }

    pub async fn read_characteristic(
        &self,
        device_name: String,
        characteristic_id: Uuid,
    ) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::ReadCharacteristic(
                device_name,
                characteristic_id,
                tx,
            ))
            .expect("Failed to send message to Bluetooth actor");
        rx.await
            .expect("Failed to receive read characteristic response")
    }

    pub async fn write_characteristic(
        &self,
        device_name: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::WriteCharacteristic(
                device_name,
                characteristic_id,
                value,
                tx,
            ))
            .expect("Failed to send message to Bluetooth actor");

        rx.await
            .expect("Failed to receive write characteristic response")
    }

    pub async fn subscribe_to_characteristic(
        &self,
        device_name: String,
        characteristic_id: Uuid,
    ) -> Result<NotificationStream, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::SubscribeToCharacteristic(
                device_name,
                characteristic_id,
                tx,
            ))
            .expect("Failed to send message to Bluetooth actor");
        rx.await
            .expect("Failed to receive subscribe to characteristic response")
    }

    pub async fn unsubscribe_from_characteristic(
        &self,
        device_name: String,
        characteristic_id: Uuid,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::UnsubscribeFromCharacteristic(
                device_name,
                characteristic_id,
                tx,
            ))
            .expect("Failed to send message to Bluetooth actor");

        rx.await
            .expect("Failed to receive unsubscribe from characteristic response")
    }
}

pub(crate) struct BluetoothActor {
    adapter: Adapter,
    discovery: Discovery,
    connected_devices: HashMap<String, ConnectedDevice>,
}

impl BluetoothActor {
    fn new(adapter: Adapter, discovery: Discovery) -> Self {
        Self {
            adapter,
            discovery,
            connected_devices: HashMap::new(),
        }
    }

    async fn run(&mut self, mut rx: UnboundedReceiver<BluetoothMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                BluetoothMessage::SubscribeToDiscovery(result_tx) => {
                    if result_tx.send(self.discovery.subscribe().await).is_err() {
                        error!("Failed to send discovery subscription result");
                    }
                }
                BluetoothMessage::UnsubscribeFromDiscovery => self.discovery.unsubscribe().await,
                BluetoothMessage::Connect(device_id, result_tx) => {
                    if result_tx.send(self.connect(device_id).await).is_err() {
                        error!("Failed to send connect result");
                    }
                }
                BluetoothMessage::Disconnect(device_id, result_tx) => {
                    if let Err(err) = result_tx.send(self.disconnect(device_id).await) {
                        error!("Failed to send disconnect result: {:?}", err);
                    }
                }
                BluetoothMessage::ReadCharacteristic(device_name, uuid, sender) => {
                    let result = self.read_characteristic(device_name, uuid).await;
                    if sender.send(result).is_err() {
                        error!("Failed to send read characteristic result");
                    }
                }
                BluetoothMessage::WriteCharacteristic(device_name, uuid, value, sender) => {
                    let result = self.write_characteristic(device_name, uuid, value).await;
                    if sender.send(result).is_err() {
                        error!("Failed to send write characteristic result");
                    }
                }
                BluetoothMessage::SubscribeToCharacteristic(device_name, uuid, sender) => {
                    let result = self.subscribe_to_characteristic(device_name, uuid).await;
                    if sender.send(result).is_err() {
                        error!("Failed to send subscribe to characteristic result");
                    }
                }
                BluetoothMessage::UnsubscribeFromCharacteristic(device_name, uuid, sender) => {
                    let result = self
                        .unsubscribe_from_characteristic(device_name, uuid)
                        .await;
                    if sender.send(result).is_err() {
                        error!("Failed to send unsubscribe from characteristic result");
                    }
                }
            }
        }
    }

    async fn connect(&mut self, device_name: String) -> Result<DeviceData, Error> {
        if let Some(device) = self.connected_devices.get_mut(&device_name) {
            info!("Reusing existing connection to {}", device_name);

            device.add_client();

            return Ok((&*device).into());
        }

        let peripherals = self.adapter.peripherals().await?;

        for peripheral in peripherals {
            let properties = peripheral.properties().await?;
            if let Some(properties) = properties {
                if let Some(name) = properties.local_name {
                    if name == device_name {
                        info!("Found device: {}", device_name);

                        // Connect to the peripheral
                        peripheral.connect().await?;
                        info!("Connected to {}", device_name);

                        let mut connected_device =
                            ConnectedDevice::start(peripheral.clone(), device_name.clone()).await?;

                        connected_device.add_client();

                        let device_data: DeviceData = (&connected_device).into();

                        self.connected_devices.insert(device_name, connected_device);

                        return Ok(device_data);
                    }
                }
            }
        }

        Err(Error::DeviceNotFound)
    }

    async fn disconnect(&mut self, name: String) -> Result<(), Error> {
        if let Some(device) = self.connected_devices.get_mut(&name) {
            device.remove_client();

            if device.has_no_clients() {
                let device = self.connected_devices.remove(&name);
                info!("Disconnected from {}", name);

                let device = device.unwrap();
                device.notifications.stop().await;
                device.peripheral.disconnect().await
            } else {
                Ok(())
            }
        } else {
            error!("No connected device found with ID: {}", name);

            Err(Error::DeviceNotFound)
        }
    }

    async fn characteristic(
        &self,
        device_name: String,
        uuid: Uuid,
    ) -> Result<(PlatformPeripheral, Characteristic), Error> {
        let peripheral = self
            .connected_devices
            .get(&device_name)
            .ok_or(Error::DeviceNotFound)?
            .peripheral
            .clone();

        let characteristic = peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == uuid)
            .ok_or(Error::NoSuchCharacteristic)?;

        Ok((peripheral, characteristic))
    }

    async fn read_characteristic(&self, device_name: String, uuid: Uuid) -> Result<Vec<u8>, Error> {
        let (peripheral, characteristic) = self.characteristic(device_name, uuid).await?;
        peripheral.read(&characteristic).await
    }

    async fn write_characteristic(
        &self,
        device_name: String,
        uuid: Uuid,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        let (peripheral, characteristic) = self.characteristic(device_name, uuid).await?;
        peripheral
            .write(
                &characteristic,
                &value,
                btleplug::api::WriteType::WithResponse,
            )
            .await
    }

    async fn subscribe_to_characteristic(
        &mut self,
        device_name: String,
        uuid: Uuid,
    ) -> Result<NotificationStream, Error> {
        let device = self
            .connected_devices
            .get_mut(&device_name)
            .ok_or(Error::DeviceNotFound)?;

        let notification_stream = device.notifications.subscribe(uuid).await?;

        Ok(notification_stream)
    }

    async fn unsubscribe_from_characteristic(
        &mut self,
        device_name: String,
        uuid: Uuid,
    ) -> Result<(), Error> {
        let device = self
            .connected_devices
            .get_mut(&device_name)
            .ok_or(Error::DeviceNotFound)?;

        device.notifications.unsubscribe(uuid).await
    }
}

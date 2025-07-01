use std::collections::HashMap;

use btleplug::{
    api::{Central, Manager as _, Peripheral},
    platform::{Adapter, Manager},
    Error,
};
use connected_device::ConnectedDevice;
use device_data::DeviceData;
use device_id::DeviceId;
use discovery::discovery_stream::DiscoveryStream;
use log::{error, info};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::{self},
};

use crate::bluetooth::discovery::discovered_device::DiscoveredDevice;

use self::discovery::Discovery;

pub mod connected_device;
pub mod device_data;
pub mod device_id;
pub mod discovery;

enum BluetoothMessage {
    SubscribeToDiscovery(oneshot::Sender<Result<DiscoveryStream, Error>>),
    UnsubscribeFromDiscovery,
    Connect(DeviceId, oneshot::Sender<Result<DeviceData, Error>>),
    Disconnect(DeviceId),
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

    pub async fn connect(&self, device_id: DeviceId) -> Result<DeviceData, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::Connect(device_id, tx))
            .expect("Failed to send message to Bluetooth actor");

        rx.await.expect("Failed to receive connect response")
    }

    pub async fn disconnect(&self, device_id: DeviceId) {
        self.tx
            .send(BluetoothMessage::Disconnect(device_id))
            .expect("Failed to send message to Bluetooth actor");
    }
}

pub(crate) struct BluetoothActor {
    adapter: Adapter,
    discovery: Discovery,
    connected_devices: HashMap<DeviceId, ConnectedDevice>,
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
                    if let Err(_) = result_tx.send(self.discovery.subscribe().await) {
                        error!("Failed to send discovery subscription result");
                    }
                }
                BluetoothMessage::UnsubscribeFromDiscovery => self.discovery.unsubscribe().await,
                BluetoothMessage::Connect(device_id, result_tx) => {
                    if let Err(_) = result_tx.send(self.connect(device_id).await) {
                        error!("Failed to send connect result");
                    }
                }
                BluetoothMessage::Disconnect(device_id) => {
                    self.disconnect(device_id).await;
                }
            }
        }
    }

    async fn connect(&mut self, device_id: String) -> Result<DeviceData, Error> {
        if let Some(device) = self.connected_devices.get_mut(&device_id) {
            info!("Reusing existing connection to {}", device_id);

            device.add_client();

            return Ok((&*device).into());
        }

        let peripherals = self.adapter.peripherals().await?;

        for peripheral in peripherals {
            let properties = peripheral.properties().await?;
            if let Some(properties) = properties {
                if let Some(name) = properties.local_name {
                    if name == device_id {
                        info!("Found device: {}", device_id);

                        // Connect to the peripheral
                        peripheral.connect().await?;
                        let properties = peripheral
                            .properties()
                            .await?
                            .ok_or(Error::DeviceNotFound)?;
                        let discovered_device: DiscoveredDevice =
                            (device_id.clone(), properties).into();
                        peripheral.discover_services().await?;
                        let services = peripheral.services();

                        // services BTree as HashMap
                        let services: HashMap<_, _> = services
                            .into_iter()
                            .map(|s| (s.uuid.to_string(), s))
                            .collect();
                        /*
                        let notification_stream = peripheral.notifications().await?;

                        let notification_abort =
                            Notifications::start(device_id.clone(), notification_stream));

                        self.notification_aborts
                            .insert(device_id.clone(), notification_abort);
                            */

                        info!("Connected to {}", device_id);

                        let mut connected_device =
                            ConnectedDevice::new(peripheral.clone(), discovered_device, services);

                        connected_device.add_client();

                        self.connected_devices
                            .insert(device_id.clone(), connected_device.clone());

                        return Ok((&connected_device).into());
                    }
                }
            }
        }

        Err(Error::DeviceNotFound)
    }

    async fn disconnect(&mut self, device_id: DeviceId) {
        let has_no_clients = if let Some(device) = self.connected_devices.get_mut(&device_id) {
            device.remove_client();
            device.has_no_clients()
        } else {
            error!("No connected device found with ID: {}", device_id);
            false
        };

        if has_no_clients {
            let device = self.connected_devices.remove(&device_id);
            info!("Disconnected from {}", device_id);

            let device = device.unwrap();
            if let Err(e) = device.peripheral.disconnect().await {
                error!("Failed to disconnect from {}: {:?}", device_id, e);
            }
        }
    }
}

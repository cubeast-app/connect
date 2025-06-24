use btleplug::{
    api::Manager as _,
    platform::{Adapter, Manager},
    Error,
};
use connected_device::ConnectedDevice;
use device_id::DeviceId;
use discovery::discovery_stream::DiscoveryStream;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::{self},
};

use self::discovery::Discovery;

pub mod connected_device;
pub mod device_id;
pub mod discovery;

enum BluetoothMessage {
    SubscribeToDiscovery(oneshot::Sender<Result<DiscoveryStream, Error>>),
    UnsubscribeFromDiscovery,
    Connect(DeviceId, oneshot::Sender<Result<ConnectedDevice, Error>>),
    Disconnect(DeviceId, oneshot::Sender<Result<(), Error>>),
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

    pub async fn connect(&self, device_id: DeviceId) -> Result<ConnectedDevice, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::Connect(device_id, tx))
            .expect("Failed to send message to Bluetooth actor");

        rx.await.expect("Failed to receive connect response")
    }

    pub async fn disconnect(&self, device_id: DeviceId) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(BluetoothMessage::Disconnect(device_id, tx))
            .expect("Failed to send message to Bluetooth actor");

        rx.await.expect("Failed to receive disconnect response")
    }
}

pub(crate) struct BluetoothActor {
    adapter: Adapter,
    discovery: Discovery,
}

impl BluetoothActor {
    fn new(adapter: Adapter, discovery: Discovery) -> Self {
        Self { adapter, discovery }
    }

    async fn run(&mut self, mut rx: UnboundedReceiver<BluetoothMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                BluetoothMessage::SubscribeToDiscovery(result_tx) => {
                    result_tx.send(self.discovery.subscribe().await);
                }
                BluetoothMessage::UnsubscribeFromDiscovery => self.discovery.unsubscribe().await,
                BluetoothMessage::Connect(device_id, result_tx) => todo!(),
                BluetoothMessage::Disconnect(device_id, result_tx) => {
                    todo!()
                }
            }
        }
    }
}

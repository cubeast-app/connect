use std::collections::HashMap;

use futures_util::SinkExt;
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log::{error, info, warn};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;
use tokio::{net::TcpStream, select, sync::mpsc::UnboundedSender};
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage};
use uuid::Uuid;

use crate::server::message::response::Response;
use crate::{
    bluetooth::characteristic_value::CharacteristicValue, server::message::broadcast::Broadcast,
};
use crate::{
    bluetooth::notifications::notification_stream::NotificationStream,
    server::message::request::Request,
};
use crate::{
    bluetooth::{
        discovery::{discovered_device::DiscoveredDevice, discovery_stream::DiscoveryStream},
        Bluetooth,
    },
    server::message::Message,
};

#[derive(Debug)]
pub(super) enum ConnectionMessage {
    /// Message received from the websocket client associated with this connection
    WebsocketMessageReceived(Result<TungsteniteMessage, TungsteniteError>),
    /// An update from Bluetooth discovery
    DevicesDiscovered(Vec<DiscoveredDevice>),
    CharacteristicNotification {
        device_id: String,
        characteristic_id: Uuid,
        value: CharacteristicValue,
    },
}

pub(super) struct ConnectionActor {
    bluetooth: Bluetooth,
    self_tx: UnboundedSender<ConnectionMessage>,
    websocket_write: SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
    discovery_abort: Option<oneshot::Sender<()>>,
    notification_aborts: HashMap<(String, Uuid), oneshot::Sender<()>>,
}

impl ConnectionActor {
    pub fn new(
        bluetooth: Bluetooth,
        self_tx: UnboundedSender<ConnectionMessage>,
        write: SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
    ) -> Self {
        Self {
            bluetooth,
            self_tx,
            websocket_write: write,
            discovery_abort: None,
            notification_aborts: HashMap::new(),
        }
    }

    pub(super) async fn run(&mut self, mut rx: UnboundedReceiver<ConnectionMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                ConnectionMessage::WebsocketMessageReceived(message) => {
                    self.websocket_message(message).await
                }
                ConnectionMessage::DevicesDiscovered(devices) => {
                    self.devices_discovered(devices).await
                }
                ConnectionMessage::CharacteristicNotification {
                    device_id,
                    characteristic_id,
                    value,
                } => {
                    self.characteristic_notification(device_id, characteristic_id, value)
                        .await
                }
            }
        }
    }

    async fn websocket_message(&mut self, message: Result<TungsteniteMessage, TungsteniteError>) {
        let response_message = if let Ok(TungsteniteMessage::Text(text)) = &message {
            let message: Result<Message, _> = serde_json::from_str(text);

            match message {
                Ok(Message::Request { request, id }) => {
                    let response = self.request(request).await;
                    Message::Response { response, id }
                }
                Ok(_) => Message::Error {
                    message: "Request expected".to_owned(),
                },

                Err(error) => Message::Error {
                    message: format!("Invalid message format or type: {error:?}"),
                },
            }
        } else {
            Message::Error {
                message: "Invalid message format".to_owned(),
            }
        };

        let serialized = serde_json::to_string(&response_message).unwrap();
        if let Err(err) = self
            .websocket_write
            .send(TungsteniteMessage::Text(serialized))
            .await
        {
            error!("Failed to send websocket message: {:?}", err);
        }
    }

    async fn devices_discovered(&mut self, devices: Vec<DiscoveredDevice>) {
        let broadcast = Broadcast::DiscoveredDevices { devices };

        let serialized = serde_json::to_string(&broadcast).unwrap();
        let broadcast = TungsteniteMessage::Text(serialized);

        let result = self.websocket_write.send(broadcast).await;

        if let Err(err) = result {
            warn!("Failed to send message to client: {:?}", err);
        }
    }

    async fn characteristic_notification(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
        value: CharacteristicValue,
    ) {
        let broadcast = Broadcast::CharacteristicValue {
            device_id,
            characteristic_id,
            value: value.value,
            timestamp: value.timestamp,
        };

        let serialized = serde_json::to_string(&broadcast).unwrap();
        let broadcast = TungsteniteMessage::Text(serialized);

        if let Err(err) = self.websocket_write.send(broadcast).await {
            warn!("Failed to send characteristic notification: {:?}", err);
        }
    }

    async fn request(&mut self, request: Request) -> Response {
        match request {
            Request::StartDiscovery => self.start_discovery().await,
            Request::StopDiscovery => self.stop_discovery(),
            Request::Connect { device_id } => {
                let result = self.bluetooth.connect(&device_id).await;

                match result {
                    Ok(device) => Response::Connected { device },
                    Err(error) => Response::Error {
                        error: format!("Failed to connect to device: {error:?}"),
                    },
                }
            }
            Request::Disconnect { device_id } => {
                match self.bluetooth.disconnect(&device_id).await {
                    Ok(()) => Response::Ok,
                    Err(error) => Response::Error {
                        error: format!("Failed to disconnect from device: {error:?}"),
                    },
                }
            }

            Request::ReadCharacteristic {
                device_id,
                characteristic_id,
            } => {
                let result = self
                    .bluetooth
                    .read_characteristic(&device_id, characteristic_id)
                    .await;

                match result {
                    Ok(value) => Response::Value {
                        value: value.value,
                        timestamp: value.timestamp,
                    },
                    Err(error) => Response::Error {
                        error: format!("Failed to read characteristic: {error:?}"),
                    },
                }
            }
            Request::WriteCharacteristic {
                device_id,
                characteristic_id,
                value,
            } => {
                self.write_characteristic(device_id, characteristic_id, value)
                    .await
            }
            Request::SubscribeToCharacteristic {
                device_id,
                characteristic_id,
            } => {
                self.subscribe_to_characteristic(device_id, characteristic_id)
                    .await
            }
            Request::UnsubscribeFromCharacteristic {
                device_id,
                characteristic_id,
            } => {
                self.unsubscribe_from_characteristic(device_id, characteristic_id)
                    .await
            }

            Request::Version => Response::Version {
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    async fn write_characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    ) -> Response {
        let result = self
            .bluetooth
            .write_characteristic(&device_id, characteristic_id, value)
            .await;
        if result.is_ok() {
            Response::Ok
        } else {
            Response::Error {
                error: String::from("Failed to write characteristic"),
            }
        }
    }

    fn stop_discovery(&mut self) -> Response {
        if let Some(discovery_abort) = self.discovery_abort.take() {
            let result = discovery_abort.send(());

            if let Err(err) = result {
                error!("Failed to abort discovery: {:?}", err);
            }

            Response::Ok
        } else {
            Response::error("Discovery is not running")
        }
    }

    pub(super) fn websocket(&self, mut read: SplitStream<WebSocketStream<TcpStream>>) {
        let tx = self.self_tx.clone();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                if let Err(err) = tx.send(ConnectionMessage::WebsocketMessageReceived(message)) {
                    error!("Failed to send websocket message: {:?}", err);
                }
            }

            info!("Websocket message stream ended");
        });
    }

    async fn start_discovery(&mut self) -> Response {
        let result = self.bluetooth.subscribe_to_discovery().await;

        match result {
            Ok(discovery_stream) => {
                let (abort_sender, abort_receiver) = oneshot::channel();

                self.devices_discovered_stream(discovery_stream, abort_receiver);

                self.discovery_abort = Some(abort_sender);

                Response::Ok
            }
            Err(err) => {
                error!("Failed to start discovery: {:?}", err);
                Response::Error {
                    error: String::from("Failed to start discovery"),
                }
            }
        }
    }

    pub(crate) fn devices_discovered_stream(
        &self,
        mut discovery_stream: DiscoveryStream,
        abort: oneshot::Receiver<()>,
    ) {
        let tx = self.self_tx.clone();

        tokio::spawn(async move {
            use futures_util::FutureExt;
            let mut abort = Box::pin(abort).fuse();

            loop {
                select! {
                        _ = (&mut abort) => {
                            break;
                        },
                        devices = discovery_stream.next() => {
                            if let Some(devices) = devices {
                                if let Err(err) = tx.send(ConnectionMessage::DevicesDiscovered(devices)) {
                                    error!("Failed to send discovered devices: {:?}", err);
                                }
                            } else {
                                break;
                            }
                        },
                }
            }
        });
    }

    async fn subscribe_to_characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
    ) -> Response {
        let result = self
            .bluetooth
            .subscribe_to_characteristic(&device_id, characteristic_id)
            .await;

        match result {
            Ok(notification_stream) => {
                let abort_sender = self.notification_stream(
                    notification_stream,
                    device_id.clone(),
                    characteristic_id,
                );

                self.notification_aborts
                    .insert((device_id, characteristic_id), abort_sender);

                Response::Ok
            }
            Err(err) => Response::Error {
                error: format!("Failed to subscribe to characteristic: {err:?}"),
            },
        }
    }

    async fn unsubscribe_from_characteristic(
        &mut self,
        device_id: String,
        characteristic_id: Uuid,
    ) -> Response {
        let result = self
            .bluetooth
            .unsubscribe_from_characteristic(&device_id, characteristic_id)
            .await;
        if result.is_ok() {
            if let Some(abort_sender) = self
                .notification_aborts
                .remove(&(device_id, characteristic_id))
            {
                let _ = abort_sender.send(());
            }

            Response::Ok
        } else {
            Response::Error {
                error: String::from("Failed to unsubscribe characteristic"),
            }
        }
    }

    pub(crate) fn notification_stream(
        &self,
        mut notification_stream: NotificationStream,
        device_id: String,
        characteristic_id: Uuid,
    ) -> oneshot::Sender<()> {
        let (abort_sender, abort_receiver) = oneshot::channel();
        let tx = self.self_tx.clone();

        tokio::spawn(async move {
            use futures_util::FutureExt;
            let mut abort = Box::pin(abort_receiver).fuse();

            loop {
                select! {
                    _ = (&mut abort) => {
                        break;
                    },
                    notification = notification_stream.next() => {
                        if let Some(value) = notification {
                            let timestamp = chrono::Utc::now().timestamp_millis() as u64;
                            if let Err(err) = tx.send(ConnectionMessage::CharacteristicNotification {
                                device_id: device_id.clone(),
                                characteristic_id,
                                value: CharacteristicValue { timestamp, value: value.value },
                            }) {
                                error!("Failed to send notification: {:?}", err);
                            }
                        } else {
                            break;
                        }
                    },
                }
            }
        });

        abort_sender
    }
}

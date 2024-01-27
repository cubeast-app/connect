use futures_util::stream::SplitSink;
use futures_util::{select, FutureExt, SinkExt, StreamExt};
use log::{error, warn};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage};

use crate::bluetooth::discovery::discovered_device::DiscoveredDevice;
use crate::bluetooth::discovery::discovery_stream::DiscoveryStream;
use crate::bluetooth::Bluetooth;
use crate::server::message::broadcast::Broadcast;
use crate::server::message::request::Request;
use crate::server::message::response::Response;
use crate::version::VERSION;

#[derive(Debug)]
pub(super) enum ConnectionMessage {
    WebsocketMessage(Result<TungsteniteMessage, TungsteniteError>),
    Discovery(Vec<DiscoveredDevice>),
    Stop,
}

pub(super) struct ConnectionActor {
    bluetooth: Bluetooth,
    sender: Sender<ConnectionMessage>,
    websocket_write: SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
    discovery_abort: Option<oneshot::Sender<()>>,
}

impl ConnectionActor {
    pub fn new(
        bluetooth: Bluetooth,
        sender: Sender<ConnectionMessage>,
        write: SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
    ) -> Self {
        Self {
            bluetooth,
            sender,
            websocket_write: write,
            discovery_abort: None,
        }
    }

    pub(super) async fn run(&mut self, mut rx: Receiver<ConnectionMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                ConnectionMessage::WebsocketMessage(message) => {
                    self.websocket_message(message).await
                }
                ConnectionMessage::Discovery(devices) => self.discovery(devices).await,
                ConnectionMessage::Stop => todo!(),
            }
        }
    }

    async fn websocket_message(&mut self, message: Result<TungsteniteMessage, TungsteniteError>) {
        let response = if let Ok(TungsteniteMessage::Text(text)) = &message {
            let request: Result<Request, _> = serde_json::from_str(&text);

            if let Ok(request) = request {
                Some(self.handle_request(request).await)
            } else {
                None
            }
        } else {
            None
        };

        let response = response.unwrap_or_else(|| {
            warn!("Received invalid message from client: {:?}", message);

            Response::Error {
                error: String::from("Invalid message"),
            }
        });

        let serialized = serde_json::to_string(&response).unwrap();
        if let Err(err) = self
            .websocket_write
            .send(TungsteniteMessage::Text(serialized))
            .await
        {
            error!("Failed to send websocket message: {:?}", err);
        }
    }

    async fn handle_request(&mut self, request: Request) -> Response {
        match request {
            Request::StartDiscovery => {
                let result = self.bluetooth.discovery.subscribe().await;

                if let Ok(response_receiver) = result {
                    let response = response_receiver.await;
                    match response {
                        Ok(stream) => {
                            let abort = self.discovery_stream(stream).await;
                            self.discovery_abort = Some(abort);
                            Response::Ok
                        }
                        Err(_) => Response::error("Failed to start discovery"),
                    }
                } else {
                    Response::error("Failed to start discovery")
                }
            }
            Request::Authenticate => todo!(),
            Request::StopDiscovery => {
                if let Some(discovery_abort) = self.discovery_abort.take() {
                    let result = self.bluetooth.discovery.unsubscribe().await;

                    if result.is_err() {
                        error!("Failed to stop discovery");
                    }

                    let result = discovery_abort.send(());

                    if let Err(err) = result {
                        error!("Failed to abort discovery: {:?}", err);
                    }

                    self.discovery_abort = None;

                    Response::Ok
                } else {
                    Response::error("Discovery is not running")
                }
            }
            /*
            Request::Connect { id: name } => {
                let result = self
                    .controller
                    .connect_client_to_device(name, self.client_id)
                    .await;

                if let Ok(discovered_device) = result {
                    Response::Connected {
                        device: discovered_device.0,
                        services: discovered_device.1,
                    }
                } else {
                    Response::Error {
                        error: String::from("Failed to connect"),
                    }
                }
            }
            Request::Disconnect { id: name } => {
                let result = self
                    .controller
                    .disconnect_client_from_device(name, self.client_id)
                    .await;

                if result.is_ok() {
                    Response::Ok
                } else {
                    Response::Error {
                        error: String::from("Failed to disconnect"),
                    }
                }
            }
            Request::ReadCharacteristic {
                device_id,
                characteristic_id,
            } => {
                let result = self
                    .controller
                    .read_characteristic(device_id, characteristic_id)
                    .await;

                if let Ok(value) = result {
                    Response::Value { value }
                } else {
                    Response::Error {
                        error: String::from("Failed to read characteristic"),
                    }
                }
            }
            Request::WriteCharacteristic {
                device_id,
                characteristic_id,
                value,
            } => {
                let result = self
                    .controller
                    .write_characteristic(device_id, characteristic_id, value)
                    .await;

                if result.is_ok() {
                    Response::Ok
                } else {
                    Response::Error {
                        error: String::from("Failed to write characteristic"),
                    }
                }
            }
            Request::SubscribeCharacteristic {
                device_id,
                characteristic_id,
            } => {
                let result = self
                    .controller
                    .subscribe_characteristic(self.client_id, device_id, characteristic_id)
                    .await;

                if result.is_ok() {
                    Response::Ok
                } else {
                    Response::Error {
                        error: String::from("Failed to subscribe characteristic"),
                    }
                }
            }
            Request::UnsubscribeCharacteristic {
                device_id,
                characteristic_id,
            } => {
                let result = self
                    .controller
                    .unsubscribe_characteristic(device_id, characteristic_id, &self.client_id)
                    .await;

                if result.is_ok() {
                    Response::Ok
                } else {
                    Response::Error {
                        error: String::from("Failed to unsubscribe characteristic"),
                    }
                }
            }
            */
            Request::Version => Response::Version { version: VERSION },
        }
    }

    async fn discovery(&mut self, devices: Vec<DiscoveredDevice>) {
        let broadcast = Broadcast::DiscoveredDevices { devices };

        let serialized = serde_json::to_string(&broadcast).unwrap();
        let broadcast = TungsteniteMessage::Text(serialized);

        let result = self.websocket_write.send(broadcast).await;

        if let Err(err) = result {
            warn!("Failed to send message to client: {:?}", err);
        }
    }

    async fn discovery_stream(&self, mut stream: DiscoveryStream) -> oneshot::Sender<()> {
        let (tx, rx) = oneshot::channel::<()>();
        let sender = self.sender.clone();
        let mut rx = rx.fuse();

        tokio::spawn(async move {
            loop {
                select! {
                    _ = &mut rx => break,
                    devices = stream.next().fuse() => {
                        match devices {
                            Some(devices) => {
                                let result = sender.send(ConnectionMessage::Discovery(devices));

                                if let Err(error) = result.await {
                                    error!("Failed to send discovery message: {:?}", error);
                                }
                            },
                            None => break,
                        }
                    }
                }
            }
        });

        tx
    }
}

use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage};
use uuid::Uuid;

use crate::controller::Controller;
use crate::version::VERSION;

use super::message::request::Request;
use super::message::response::Response;
use super::message::Message;

pub struct ServerConnection {
    controller: Controller,
    write: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>>>,
    client_id: Uuid,
}

impl ServerConnection {
    pub fn start(
        read: SplitStream<WebSocketStream<TcpStream>>,
        write: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>>>,
        controller: Controller,
        client_id: Uuid,
    ) {
        let connection = Self {
            controller,
            write,
            client_id,
        };

        tokio::spawn(async move { connection.run(read).await });
    }

    async fn run(&self, read: SplitStream<WebSocketStream<TcpStream>>) {
        info!("Connected client {}", self.client_id);

        let json_messages = read.filter_map(|message| async {
            if let Ok(message) = message {
                if let TungsteniteMessage::Text(text) = message {
                    let message: Option<Message> = serde_json::from_str(&text).ok();
                    Some(message)
                } else {
                    None
                }
            } else {
                warn!("Received invalid message from client: {:?}", message);
                None
            }
        });

        let responses = json_messages.filter_map(|message| async {
            let response = self.handle_message(message).await;
            let serialized = serde_json::to_string(&response).unwrap();
            Some(TungsteniteMessage::Text(serialized))
        });

        responses
            .for_each(|response| async {
                let mut sink = self.write.lock().await;
                sink.send(response).await.unwrap();
            })
            .await;

        info!("Disconnected client {}", self.client_id);

        self.controller
            .remove_client(&self.client_id)
            .await
            .expect("Failed to remove client");
    }

    async fn handle_message(&self, message: Option<Message>) -> Message {
        if let Some(Message::Request { id, request }) = message {
            let response = self.handle_request(request).await;
            Message::Response { id, response }
        } else if message.is_some() {
            Message::Error {
                message: String::from("Request expected"),
            }
        } else {
            Message::Error {
                message: String::from("Invalid message"),
            }
        }
    }

    async fn handle_request(&self, request: Request) -> Response {
        match request {
            Request::StartDiscovery => {
                let result = self.controller.add_discovery_client(self.client_id).await;

                if result.is_ok() {
                    Response::Ok
                } else {
                    Response::Error {
                        error: String::from("Failed to start discovery"),
                    }
                }
            }
            Request::Authenticate => todo!(),
            Request::StopDiscovery => {
                let result = self
                    .controller
                    .remove_discovery_client(&self.client_id)
                    .await;

                if result.is_ok() {
                    Response::Ok
                } else {
                    Response::Error {
                        error: String::from("Failed to stop discovery"),
                    }
                }
            }
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
            Request::Version => Response::Version { version: VERSION },
        }
    }
}

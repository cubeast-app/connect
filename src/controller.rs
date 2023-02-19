use std::{collections::HashSet, sync::Arc, time::Duration};

use btleplug::{
    api::{Central, Peripheral},
    Error,
};
use log::info;
use tokio::{
    sync::{
        mpsc::{error::SendError, Sender},
        Mutex,
    },
    time,
};
use uuid::Uuid;

use crate::{
    adapter::bluetooth_adapter,
    app_state::{AppState, ConnectedDevice},
    broadcaster::broadcast_command::BroadcastCommand,
    clients::Client,
    discovery::discovery_command::DiscoveryCommand,
    server::message::broadcast::{Broadcast, DiscoveredDevice},
};

const STOP_DISCOVERY_TIMEOUT: u64 = 60000;

#[derive(Clone)]
pub struct Controller {
    app_state: Arc<Mutex<AppState>>,
    discovery_tx: Sender<DiscoveryCommand>,
    broadcaster_tx: Sender<BroadcastCommand>,
}

impl Controller {
    pub fn new(
        app_state: Arc<Mutex<AppState>>,
        discovery_tx: Sender<DiscoveryCommand>,
        broadcaster_tx: Sender<BroadcastCommand>,
    ) -> Controller {
        Controller {
            app_state,
            discovery_tx,
            broadcaster_tx,
        }
    }

    pub async fn add_client(&self, client: Client) -> Uuid {
        let mut app_state = self.app_state.lock().await;
        app_state.clients.add(client).await
    }

    pub async fn add_discovery_client(&self, client_id: Uuid) {
        let mut app_state = self.app_state.lock().await;

        let was_empty = app_state.discovery_clients.is_empty();

        app_state.discovery_clients.insert(client_id);

        if was_empty {
            self.discovery_tx
                .send(DiscoveryCommand::Start)
                .await
                .expect("Failed to send discovery command");

            let app_state = self.app_state.clone();
            let discovery_tx = self.discovery_tx.clone();
            let broadcaster_tx = self.broadcaster_tx.clone();

            tokio::spawn(async move {
                time::sleep(Duration::from_millis(STOP_DISCOVERY_TIMEOUT)).await;
                Controller::new(app_state, discovery_tx, broadcaster_tx)
                    .remove_discovery_client(client_id)
                    .await;
            });
        }
    }

    pub async fn remove_discovery_client(&self, client_id: Uuid) {
        let mut app_state = self.app_state.lock().await;

        app_state.discovery_clients.remove(&client_id);

        if app_state.discovery_clients.is_empty() {
            self.discovery_tx
                .send(DiscoveryCommand::Stop)
                .await
                .expect("Failed to send discovery command");
        }
    }

    pub async fn connect(&self, id: String, client_id: Uuid) -> Result<(), Error> {
        let adapter = bluetooth_adapter().await?;
        let peripherals = adapter.peripherals().await?;
        let peripheral = peripherals.iter().find(|p| p.id().to_string() == id);

        if let Some(peripheral) = peripheral {
            let mut app_state = self.app_state.lock().await;

            if let Some(connected_device) = app_state.connected_devices.get_mut(&id) {
                connected_device.clients.insert(client_id);
            } else {
                peripheral.connect().await?;

                info!("Connected to {}", id);

                let mut clients = HashSet::new();
                clients.insert(client_id);

                app_state.connected_devices.insert(
                    id.clone(),
                    ConnectedDevice {
                        clients,
                        peripheral: peripheral.clone(),
                    },
                );
            }

            Ok(())
        } else {
            Err(Error::DeviceNotFound)
        }
    }

    pub async fn disconnect(&self, id: String, client_id: Uuid) -> Result<(), Error> {
        let mut app_state = self.app_state.lock().await;

        if let Some(connected_device) = app_state.connected_devices.get_mut(&id) {
            connected_device.clients.remove(&client_id);

            if connected_device.clients.is_empty() {
                connected_device.peripheral.disconnect().await?;
                app_state.connected_devices.remove(&id);

                info!("Disconnected from {}", id);
            }
        }

        Ok(())
    }

    pub(crate) async fn update_discovered_devices(
        &self,
        discovered_devices: Vec<DiscoveredDevice>,
    ) -> Result<(), SendError<BroadcastCommand>> {
        let app_state = self.app_state.lock().await;
        let discovery_clients = app_state
            .clients
            .get_by_ids(
                &(app_state
                    .discovery_clients
                    .clone()
                    .into_iter()
                    .collect::<Vec<_>>()),
            )
            .await;
        self.broadcaster_tx
            .send(BroadcastCommand {
                broadcast: Broadcast::DiscoveredDevices {
                    devices: discovered_devices,
                },
                clients: discovery_clients,
            })
            .await
    }
}

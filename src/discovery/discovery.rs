use btleplug::api::{Central, CentralEvent, Peripheral, ScanFilter};
use btleplug::platform::Adapter;
use btleplug::Error;
use futures_util::StreamExt;
use log::{info, warn};
use tokio::select;
use tokio::sync::mpsc::Receiver;

use crate::adapter::bluetooth_adapter;
use crate::controller::Controller;
use crate::server::message::broadcast::DiscoveredDevice;

use super::discovery_command::DiscoveryCommand;

pub struct Discovery {
    controller: Controller,
    discovery_rx: Receiver<DiscoveryCommand>,
    last_devices: Vec<DiscoveredDevice>,
}

impl<'controller> Discovery {
    fn new(controller: Controller, discovery_rx: Receiver<DiscoveryCommand>) -> Self {
        Self {
            controller,
            discovery_rx,
            last_devices: vec![],
        }
    }

    pub async fn start(controller: Controller, discovery_rx: Receiver<DiscoveryCommand>) {
        let adapter = bluetooth_adapter()
            .await
            .expect("Cubeast Connect was unable to connect to Bluetooth");

        info!("Using adapter {:?}", adapter.adapter_info().await.unwrap());

        let mut discovery = Discovery::new(controller, discovery_rx);

        tokio::spawn(async move {
            discovery.run(adapter).await;
        });
    }

    async fn run(&mut self, adapter: Adapter) {
        let mut events = adapter.events().await.unwrap();

        loop {
            select! {
                command = self.discovery_rx.recv() => {
                    if let Some(command) = command {
                        self.handle_client_command(&adapter, command).await;
                    }
                },
                event = events.next() => {
                    if let Some(event) = event {
                        if let Err(err) = self.handle_central_event(event, &adapter).await {
                            warn!("Error handling central event: {:?}", err)
                        }
                    }
                },
            }
        }
    }

    async fn handle_client_command(&mut self, adapter: &Adapter, command: DiscoveryCommand) {
        match command {
            DiscoveryCommand::Start => {
                adapter.start_scan(ScanFilter::default()).await.unwrap();
                info!("Started discovery");
            }
            DiscoveryCommand::Stop => {
                if let Err(error) = adapter.stop_scan().await {
                    warn!("Error stopping discovery: {:?}", error);
                } else {
                    info!("Stopped discovery");
                }
            }
        }
    }

    async fn handle_central_event(
        &mut self,
        _event: CentralEvent,
        adapter: &Adapter,
    ) -> Result<(), Error> {
        let peripherals = adapter.peripherals().await?;

        let mut discovered_devices = vec![];

        for peripheral in peripherals {
            let properties = peripheral.properties().await?;
            let id = peripheral.id().to_string();

            let device = if let Some(properties) = properties {
                DiscoveredDevice {
                    id,
                    name: properties.local_name,
                    signal_strength: properties.rssi,
                    address: Some(properties.address.to_string()),
                    manufacturer_data: Some(properties.manufacturer_data),
                }
            } else {
                DiscoveredDevice {
                    id,
                    name: None,
                    signal_strength: None,
                    address: None,
                    manufacturer_data: None,
                }
            };

            discovered_devices.push(device);
        }

        discovered_devices.sort_by(|a, b| a.name.cmp(&b.name));

        if self.last_devices.eq(&discovered_devices) {
            return Ok(());
        }

        self.last_devices = discovered_devices.clone();

        self.controller
            .update_discovered_devices(discovered_devices)
            .await
            .map_err(|err| Error::Other(Box::new(err)))
    }
}

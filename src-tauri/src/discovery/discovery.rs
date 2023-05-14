use btleplug::api::{Central, CentralEvent, Peripheral, ScanFilter};
use btleplug::platform::Adapter;
use btleplug::Error;
use futures_util::{select, FutureExt, StreamExt};
use log::{info, warn};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::{channel, Sender};

use crate::discovered_device::DiscoveredDevice;
use crate::events::Events;

const CAPACITY: usize = 1;

pub struct Discovery {
    adapter: Adapter,
    events: Events,
    stop_rx: Receiver<()>,
}

impl Discovery {
    fn new(adapter: Adapter, events: Events, stop_rx: Receiver<()>) -> Self {
        Self {
            adapter,
            events,
            stop_rx,
        }
    }

    pub async fn start(adapter: Adapter, events: Events) -> Sender<()> {
        let (stop_tx, stop_rx) = channel::<()>(CAPACITY);

        info!("Using adapter {:?}", adapter.adapter_info().await.unwrap());

        tokio::spawn(async move {
            let mut discovery = Discovery::new(adapter, events, stop_rx);
            discovery.run().await;
        });

        stop_tx
    }

    async fn run(&mut self) {
        let mut events = self.adapter.events().await.unwrap();
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .unwrap();

        loop {
            select! {
                _ = self.stop_rx.recv().fuse() => {
                    info!("Stopping discovery");
                    self.adapter.stop_scan().await.unwrap();
                    return;
                },
                event = events.next().fuse() => {
                    match event {
                        Some(CentralEvent::DeviceDiscovered(_))
                        | Some(CentralEvent::DeviceUpdated(_))
                        | Some(CentralEvent::ManufacturerDataAdvertisement { .. }) => {
                            if let Err(err) = self.handle_discovery_event().await {
                                warn!("Error handling central event: {:?}", err)
                            }
                        }
                        _ => {}
                    };

                }

            }
        }
    }

    async fn handle_discovery_event(&mut self) -> Result<(), Error> {
        let peripherals = self.adapter.peripherals().await?;

        let mut discovered_devices = vec![];

        for peripheral in peripherals {
            let properties = peripheral.properties().await?;
            let id = peripheral.id().to_string();

            let device: DiscoveredDevice = if let Some(properties) = properties {
                (id, properties).into()
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

        self.events.on_discovery(discovered_devices).await;

        Ok(())
    }
}

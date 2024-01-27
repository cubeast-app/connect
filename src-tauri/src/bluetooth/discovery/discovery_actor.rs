use btleplug::{
    api::{Central as _, ScanFilter},
    platform::Adapter,
    Error,
};
use tokio::sync::mpsc::Receiver;

use super::discovery_stream::{self, DiscoveryStream};
use super::{discovered_device::DiscoveredDevice, discovery_message::DiscoveryMessage};
use log::{error, info};

pub(crate) struct DiscoveryActor {
    adapter: Adapter,
    devices: Vec<DiscoveredDevice>,
    streams: usize,
}

impl DiscoveryActor {
    pub fn new(adapter: Adapter) -> Self {
        Self {
            adapter,
            devices: vec![],
            streams: 0,
        }
    }

    pub(super) async fn run(&mut self, mut rx: Receiver<DiscoveryMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                DiscoveryMessage::Subscribe(response) => {
                    let stream = self.subscribe().await;

                    match stream {
                        Ok(stream) => {
                            if response.send(stream).is_err() {
                                error!("Failed to send discovery stream");
                            }
                        }
                        Err(error) => {
                            error!("Failed to subscribe to discovery: {:?}", error);
                        }
                    }
                }
                DiscoveryMessage::Unsubscribe(response) => {
                    self.unsubscribe().await;

                    if response.send(()).is_err() {
                        error!("Failed to send unsubscribe response");
                    }
                }
            }
        }
    }

    async fn handle_message(&self, message: DiscoveryMessage) {}

    async fn subscribe(&mut self) -> Result<DiscoveryStream, Error> {
        self.streams += 1;

        if self.streams == 1 {
            self.start_discovery().await;
        }

        discovery_stream::discovery_stream(self.adapter.clone(), self.devices.clone()).await
    }

    async fn unsubscribe(&mut self) {
        self.streams -= 1;

        if self.streams == 0 {
            self.stop_discovery().await;
        }
    }

    async fn start_discovery(&self) {
        info!("Starting discovery");
        if let Err(error) = self.adapter.start_scan(ScanFilter::default()).await {
            error!("Failed to start discovery: {:?}", error);
        }
    }

    async fn stop_discovery(&self) {
        info!("Stopping discovery");
        if let Err(error) = self.adapter.stop_scan().await {
            error!("Failed to stop discovery: {:?}", error);
        }
    }
}

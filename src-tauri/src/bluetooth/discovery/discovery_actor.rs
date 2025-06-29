use btleplug::{
    api::{Central as _, ScanFilter},
    platform::Adapter,
    Error,
};
use tokio::sync::mpsc::UnboundedReceiver;

use super::discovery_stream::{self};
use super::{
    discovered_device::DiscoveredDevice, discovery_message::DiscoveryMessage,
    discovery_stream::DiscoveryStream,
};
use log::{error, info, trace};

pub(super) struct DiscoveryActor {
    adapter: Adapter,
    devices: Vec<DiscoveredDevice>,
    subscribers_count: usize,
}

impl DiscoveryActor {
    pub(super) fn new(adapter: Adapter) -> Self {
        Self {
            adapter,
            devices: vec![],
            subscribers_count: 0,
        }
    }

    pub(super) async fn run(&mut self, mut rx: UnboundedReceiver<DiscoveryMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                DiscoveryMessage::Subscribe(tx) => {
                    let result = self.subscribe().await;

                    if tx.send(result).is_err() {
                        error!("Failed to send subscribe response");
                    }
                }
                DiscoveryMessage::Unsubscribe => self.unsubscribe().await,
            }
        }
    }

    async fn subscribe(&mut self) -> Result<DiscoveryStream, Error> {
        self.subscribers_count += 1;

        if self.subscribers_count == 1 {
            trace!("First subscriber, starting discovery");
            self.start_discovery().await?;
        }

        let stream =
            discovery_stream::discovery_stream(self.adapter.clone(), self.devices.clone()).await?;

        Ok(stream)
    }

    async fn unsubscribe(&mut self) {
        self.subscribers_count -= 1;

        if self.subscribers_count == 0 {
            trace!("No more subscribers, stopping discovery");
            self.stop_discovery()
                .await
                .expect("Failed to stop discovery");
        }
    }

    async fn start_discovery(&self) -> Result<(), Error> {
        info!("Starting discovery");
        self.adapter.start_scan(ScanFilter::default()).await
    }

    async fn stop_discovery(&self) -> Result<(), Error> {
        info!("Stopping discovery");
        self.adapter.stop_scan().await
    }
}

/*
use btleplug::Error;
use futures_util::{FutureExt, StreamExt};
use tokio::{
    select,
    sync::oneshot::{self, Sender},
};

use crate::bluetooth::{
    discovery::{discovered_device::DiscoveredDevice, discovery_stream::DiscoveryStream},
    Bluetooth,
};

#[derive(Clone)]
pub struct MainController {
    bluetooth: Bluetooth,
}

impl MainController {
    pub(crate) fn new(bluetooth: Bluetooth) -> Self {
        Self { bluetooth }
    }

    pub async fn discovery<Subscriber>(&self, subscriber: Subscriber) -> Result<Sender<()>, Error>
    where
        Subscriber: Fn(Vec<DiscoveredDevice>) + Send + 'static,
    {
        let receiver = self.bluetooth.discovery.subscribe().await;
        let response = receiver.await.expect("Failed to subscribe to discovery");

        match response {
            Ok(stream) => Ok(self.discovery_stream(stream, subscriber).await),
            Err(err) => Err(err),
        }
    }

    async fn discovery_stream<Subscriber>(
        &self,
        mut stream: DiscoveryStream,
        subscriber: Subscriber,
    ) -> Sender<()>
    where
        Subscriber: Fn(Vec<DiscoveredDevice>) + Send + 'static,
    {
        let (tx, rx) = oneshot::channel::<()>();
        let mut rx = rx.fuse();

        tokio::spawn(async move {
            loop {
                select! {
                    _ = &mut rx => break,
                    devices = stream.next().fuse() => {
                        match devices {
                            Some(devices) => {
                                subscriber(devices);
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

*/

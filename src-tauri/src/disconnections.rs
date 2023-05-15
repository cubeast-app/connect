use btleplug::{
    api::{Central, CentralEvent},
    platform::Adapter,
};
use futures_util::StreamExt;
use log::{info, warn};

use crate::events::Events;

pub struct Disconnections {}

impl Disconnections {
    pub fn start(adapter: Adapter, events: Events) {
        tokio::spawn(async move {
            Disconnections::run(adapter, events).await;
        });
    }

    pub async fn run(adapter: Adapter, events: Events) {
        let mut bluetooth_events = adapter.events().await.unwrap();

        while let Some(event) = &bluetooth_events.next().await {
            if let CentralEvent::DeviceDisconnected(device) = event {
                info!("Device disconnected: {:?}", device);

                let result = events.on_device_disconnected(device.to_string()).await;

                if result.is_err() {
                    warn!("Error disconnecting device: {:?}", result);
                }
            }
        }
    }
}

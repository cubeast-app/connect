use btleplug::{
    api::{Central, CentralEvent},
    platform::Adapter,
};
use futures_util::StreamExt;
use log::{info, warn};
use tauri::async_runtime::Sender;

pub type DisconnectionListener = Sender<String>;
pub struct Disconnections {}

impl Disconnections {
    pub fn start(adapter: Adapter, listener: DisconnectionListener) {
        tokio::spawn(async move {
            Disconnections::run(adapter, listener).await;
        });
    }

    pub async fn run(adapter: Adapter, listener: DisconnectionListener) {
        let mut bluetooth_events = adapter.events().await.unwrap();

        while let Some(event) = &bluetooth_events.next().await {
            if let CentralEvent::DeviceDisconnected(device) = event {
                info!("Device disconnected: {:?}", device);

                let result = listener.send(device.to_string()).await;

                if result.is_err() {
                    warn!("Error disconnecting device: {:?}", result);
                }
            }
        }
    }
}

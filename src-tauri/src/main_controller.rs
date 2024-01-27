use futures_util::StreamExt;
use log::error;
use tauri::{AppHandle, Manager};
use tokio::{
    sync::{mpsc::channel, mpsc::Receiver, mpsc::Sender},
    task::JoinHandle,
};

use crate::bluetooth::Bluetooth;

const BUFFER: usize = 100;

enum MainControllerMessage {
    StartDiscovery(AppHandle),
    StopDiscovery,
}

pub struct MainController {
    tx: Sender<MainControllerMessage>,
}

impl MainController {
    pub(crate) fn start(bluetooth: Bluetooth) -> Self {
        let (tx, rx) = channel(BUFFER);
        let mut actor = MainControllerActor::new(bluetooth);

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self::new(tx)
    }

    fn new(tx: Sender<MainControllerMessage>) -> Self {
        Self { tx }
    }

    pub(super) async fn start_discovery(&self, app_handle: AppHandle) -> Result<(), ()> {
        self.tx
            .send(MainControllerMessage::StartDiscovery(app_handle))
            .await
            .map_err(|_| ())
    }

    pub(super) async fn stop_discovery(&self) -> Result<(), ()> {
        self.tx
            .send(MainControllerMessage::StopDiscovery)
            .await
            .map_err(|_| ())
    }
}

pub struct MainControllerActor {
    bluetooth: Bluetooth,
    discovery_join_handle: Option<JoinHandle<()>>,
}

impl MainControllerActor {
    fn new(bluetooth: Bluetooth) -> Self {
        Self {
            bluetooth,
            discovery_join_handle: None,
        }
    }

    async fn run(&mut self, mut rx: Receiver<MainControllerMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                MainControllerMessage::StartDiscovery(app_handle) => {
                    self.start_discovery(app_handle).await;
                }
                MainControllerMessage::StopDiscovery => {
                    self.stop_discovery().await;
                }
            }
        }
    }

    async fn start_discovery(&mut self, app_handle: AppHandle) {
        if self.discovery_join_handle.is_some() {
            return;
        }

        let result = self.bluetooth.discovery.subscribe().await;

        match result {
            Ok(response) => {
                let discovery_stream = response.await;

                if let Ok(mut discovery_stream) = discovery_stream {
                    let join_handle = tokio::spawn(async move {
                        while let Some(devices) = discovery_stream.next().await {
                            if let Err(err) = app_handle.emit_all("discovery", devices) {
                                error!("Failed to emit event: {:?}", err);
                            }
                        }
                    });

                    self.discovery_join_handle = Some(join_handle);
                }
            }
            Err(_) => {
                error!("Failed to start discovery");
            }
        }
    }

    async fn stop_discovery(&mut self) {
        if let Some(join_handle) = self.discovery_join_handle.take() {
            join_handle.abort();

            let result = self.bluetooth.discovery.unsubscribe().await;

            match result {
                Ok(response) => {
                    let _ = response.await;
                }
                Err(_) => {
                    error!("Failed to unsubscribe from discovery");
                }
            }
        }
    }
}

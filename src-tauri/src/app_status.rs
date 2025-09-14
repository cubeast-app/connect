use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Status {
    CheckingForUpdates,
    DownloadingUpdate { progress: u8 },
    Running { version: String },
}

#[derive(Clone)]
pub struct AppStatus {
    current: Arc<RwLock<Status>>,
    broadcaster: broadcast::Sender<Status>,
}

impl AppStatus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000); // Buffer up to 1000 status updates
        Self {
            current: Arc::new(RwLock::new(Status::Running {
                version: env!("CARGO_PKG_VERSION").to_string(),
            })),
            broadcaster: tx,
        }
    }

    pub async fn update(&self, status: Status) {
        let previous = self.current.read().await.clone();

        if previous != status {
            let mut current = self.current.write().await;
            *current = status.clone();

            // Broadcast to all subscribers
            if let Err(err) = self.broadcaster.send(status) {
                error!("Failed to send status update: {err}");
            }
        }
    }

    pub async fn get(&self) -> Status {
        self.current.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Status> {
        self.broadcaster.subscribe()
    }
}

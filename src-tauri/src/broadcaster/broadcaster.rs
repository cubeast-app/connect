use futures_util::SinkExt;
use log::{info, warn};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage};

use crate::server::message::Message;

use super::broadcast_command::BroadcastCommand;

pub struct Broadcaster {
    broadcaster_rx: Receiver<BroadcastCommand>,
}

impl Broadcaster {
    pub fn start(broadcaster_rx: Receiver<BroadcastCommand>) {
        let mut broadcaster = Self::new(broadcaster_rx);

        tokio::spawn(async move {
            broadcaster.run().await;
        });
    }

    fn new(broadcaster_rx: Receiver<BroadcastCommand>) -> Self {
        Self { broadcaster_rx }
    }

    async fn run(&mut self) {
        info!("Starting broadcaster");

        while let Some(command) = self.broadcaster_rx.recv().await {
            info!("Broadcasting message: {}", command.broadcast);
            let serialized = serde_json::to_string(&Message::Broadcast {
                broadcast: command.broadcast,
            })
            .unwrap();

            for client in command.clients {
                let mut write = client.lock().await;
                let write_result = write
                    .send(TungsteniteMessage::Text(serialized.clone()))
                    .await;

                if write_result.is_err() {
                    warn!("Failed to broadcast a message to client");
                }
            }
        }
    }
}

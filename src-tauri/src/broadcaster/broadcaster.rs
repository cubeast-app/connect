use futures_util::SinkExt;
use log::info;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage};

use crate::server::message::Message;

use super::broadcast_command::BroadcastCommand;

pub struct Broadcaster {
    broadcaster_rx: Receiver<BroadcastCommand>,
}

impl Broadcaster {
    pub async fn start(broadcaster_rx: Receiver<BroadcastCommand>) {
        let mut broadcaster = Self::new(broadcaster_rx);

        tokio::spawn(async move {
            broadcaster.run().await;
        });
    }

    fn new(broadcaster_rx: Receiver<BroadcastCommand>) -> Self {
        Self { broadcaster_rx }
    }

    async fn run(&mut self) {
        while let Some(command) = self.broadcaster_rx.recv().await {
            let serialized = serde_json::to_string(&Message::Broadcast {
                broadcast: command.broadcast,
            })
            .unwrap();

            for (client_id, client) in command.clients {
                let mut write = client.lock().await;
                let write_result = write
                    .send(TungsteniteMessage::Text(serialized.clone()))
                    .await;

                if let Err(_) = write_result {
                    info!("Failed to broadcast a message to {}", client_id);
                }
            }
        }
    }
}

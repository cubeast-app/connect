use btleplug::{platform::Adapter, Error};
use discovery_stream::DiscoveryStream;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    oneshot::{self},
};

use self::{discovery_actor::DiscoveryActor, discovery_message::DiscoveryMessage};

pub mod discovered_device;
mod discovery_actor;
mod discovery_message;
pub mod discovery_stream;

#[derive(Clone)]
pub(crate) struct Discovery {
    tx: UnboundedSender<DiscoveryMessage>,
}

impl Discovery {
    pub fn start(adapter: Adapter) -> Self {
        let mut actor = DiscoveryActor::new(adapter);
        let (tx, rx) = unbounded_channel();

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self::new(tx)
    }

    fn new(tx: UnboundedSender<DiscoveryMessage>) -> Self {
        Self { tx }
    }

    pub async fn subscribe(&self) -> Result<DiscoveryStream, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(DiscoveryMessage::Subscribe(tx))
            .expect("Failed to send actor message");

        rx.await.expect("Failed to receive discovery stream")
    }

    pub async fn unsubscribe(&self) {
        self.tx
            .send(DiscoveryMessage::Unsubscribe)
            .expect("Failed to send actor message");
    }
}

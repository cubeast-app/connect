use btleplug::platform::Adapter;
use tokio::sync::{
    mpsc::Sender,
    mpsc::{channel, error::SendError},
    oneshot::{self, Receiver},
};

use self::{
    discovery_actor::DiscoveryActor, discovery_message::DiscoveryMessage,
    discovery_stream::DiscoveryStream,
};

pub mod discovered_device;
mod discovery_actor;
mod discovery_message;
pub mod discovery_stream;

const BUFFER: usize = 100;

#[derive(Clone)]
pub(crate) struct Discovery {
    tx: Sender<DiscoveryMessage>,
}

impl Discovery {
    pub fn start(adapter: Adapter) -> Self {
        let mut actor = DiscoveryActor::new(adapter);
        let (tx, rx) = channel(BUFFER);

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self::new(tx)
    }

    pub fn new(tx: Sender<DiscoveryMessage>) -> Self {
        Self { tx }
    }

    pub async fn subscribe(
        &self,
    ) -> Result<Receiver<DiscoveryStream>, SendError<DiscoveryMessage>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DiscoveryMessage::Subscribe(tx)).await?;

        Ok(rx)
    }

    pub async fn unsubscribe(&self) -> Result<Receiver<()>, SendError<DiscoveryMessage>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DiscoveryMessage::Unsubscribe(tx)).await?;

        Ok(rx)
    }
}

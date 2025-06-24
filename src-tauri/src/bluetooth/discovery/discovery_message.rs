use btleplug::Error;
use tokio::sync::oneshot::Sender;

use super::discovery_stream::DiscoveryStream;

pub(crate) enum DiscoveryMessage {
    Subscribe(Sender<Result<DiscoveryStream, Error>>),
    Unsubscribe,
}

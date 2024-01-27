use tokio::sync::oneshot::Sender;

use super::discovery_stream::DiscoveryStream;

pub(crate) enum DiscoveryMessage {
    Subscribe(Sender<DiscoveryStream>),
    Unsubscribe(Sender<()>),
}

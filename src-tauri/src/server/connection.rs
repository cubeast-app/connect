use futures_util::stream::{SplitSink, SplitStream};
use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use tokio_tungstenite::{tungstenite::Message as TungsteniteMessage, WebSocketStream};

use crate::bluetooth::Bluetooth;

use self::connection_actor::ConnectionActor;

mod connection_actor;

pub(crate) struct Connection {}

impl Connection {
    pub(crate) fn start(
        bluetooth: Bluetooth,
        websocket_read: SplitStream<WebSocketStream<TcpStream>>,
        websocket_write: SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
    ) -> Self {
        let (tx, rx) = unbounded_channel();
        let mut actor = ConnectionActor::new(bluetooth, tx.clone(), websocket_write);
        actor.websocket(websocket_read);

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self::new()
    }

    fn new() -> Self {
        Self {}
    }
}

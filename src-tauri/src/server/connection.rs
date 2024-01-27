use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log::error;
use tokio::{
    net::TcpStream,
    sync::{mpsc::channel, mpsc::Sender},
};
use tokio_tungstenite::{tungstenite::Message as TungsteniteMessage, WebSocketStream};

use crate::bluetooth::Bluetooth;

use self::connection_actor::{ConnectionActor, ConnectionMessage};

mod connection_actor;

const BUFFER: usize = 100;

pub(crate) struct Connection {
    tx: Sender<ConnectionMessage>,
}

impl Connection {
    pub(crate) fn start(
        bluetooth: Bluetooth,
        websocket_write: SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
    ) -> Self {
        let (tx, rx) = channel(BUFFER);
        let mut actor = ConnectionActor::new(bluetooth, tx.clone(), websocket_write);

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self::new(tx)
    }

    fn new(tx: Sender<ConnectionMessage>) -> Self {
        Self { tx }
    }

    pub(crate) async fn websocket_message(
        &self,
        message: Result<TungsteniteMessage, tokio_tungstenite::tungstenite::Error>,
    ) {
        if let Err(err) = self
            .tx
            .send(ConnectionMessage::WebsocketMessage(message))
            .await
        {
            error!("Failed to send websocket message: {:?}", err);
        }
    }

    pub(crate) fn websocket_message_stream(
        &self,
        mut read: SplitStream<WebSocketStream<TcpStream>>,
    ) {
        let tx = self.tx.clone();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                if let Err(err) = tx.send(ConnectionMessage::WebsocketMessage(message)).await {
                    error!("Failed to send websocket message: {:?}", err);
                }
            }
        });
    }
}

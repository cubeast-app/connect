use std::collections::HashMap;

use btleplug::{
    api::{CharPropFlags, Peripheral as _},
    platform::Peripheral,
    Error,
};
use notification_stream::NotificationStream;
use notifications_message::NotificationsMessage;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot,
};
use tracing::error;
use uuid::Uuid;

use crate::bluetooth::notifications::notification_stream::notification_stream;

pub mod notification_stream;
mod notifications_message;

#[derive(Clone)]
pub(crate) struct Notifications {
    tx: UnboundedSender<NotificationsMessage>,
}

impl Notifications {
    pub fn start(peripheral: Peripheral) -> Self {
        let mut actor = NotificationsActor::new(peripheral);
        let (tx, rx) = unbounded_channel();

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Self::new(tx)
    }

    fn new(tx: UnboundedSender<NotificationsMessage>) -> Self {
        Self { tx }
    }

    pub async fn subscribe(&self, characteristic_id: Uuid) -> Result<NotificationStream, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(NotificationsMessage::Subscribe(characteristic_id, tx))
            .expect("Failed to send actor message");

        rx.await.expect("Failed to receive notification stream")
    }

    pub async fn unsubscribe(&self, characteristic_id: Uuid) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(NotificationsMessage::Unsubscribe(characteristic_id, tx))
            .expect("Failed to send actor message");

        rx.await.expect("Failed to receive unsubscribe result")
    }

    pub async fn stop(&self) {
        self.tx
            .send(NotificationsMessage::Stop)
            .expect("Failed to send actor message");
    }
}

pub(super) struct NotificationsActor {
    peripheral: Peripheral,
    subscribers_count: HashMap<Uuid, usize>,
}

impl NotificationsActor {
    pub(super) fn new(peripheral: Peripheral) -> Self {
        Self {
            peripheral,
            subscribers_count: HashMap::new(),
        }
    }

    pub(super) async fn run(&mut self, mut rx: UnboundedReceiver<NotificationsMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                NotificationsMessage::Subscribe(characteristic_id, tx) => {
                    let result = self.subscribe(characteristic_id).await;
                    if tx.send(result).is_err() {
                        error!("Failed to send subscribe response");
                    }
                }
                NotificationsMessage::Unsubscribe(characteristic_id, tx) => {
                    let result = self.unsubscribe(characteristic_id).await;
                    if tx.send(result).is_err() {
                        error!("Failed to send unsubscribe response");
                    }
                }
                NotificationsMessage::Stop => break,
            }
        }
    }

    async fn subscribe(&mut self, characteristic_id: Uuid) -> Result<NotificationStream, Error> {
        let count = self.subscribers_count.entry(characteristic_id).or_insert(0);
        *count += 1;

        if *count == 1 {
            let characteristic = self
                .peripheral
                .characteristics()
                .into_iter()
                .find(|c| c.uuid == characteristic_id)
                .ok_or(Error::NoSuchCharacteristic)?;

            if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
                return Err(Error::NotSupported(
                    "Characteristic does not support notifications".to_string(),
                ));
            }

            self.peripheral.subscribe(&characteristic).await?;
        }

        notification_stream(&self.peripheral, characteristic_id).await
    }

    async fn unsubscribe(&mut self, characteristic_id: Uuid) -> Result<(), Error> {
        if let Some(count) = self.subscribers_count.get_mut(&characteristic_id) {
            *count -= 1;

            if *count == 0 {
                let characteristic = self
                    .peripheral
                    .characteristics()
                    .into_iter()
                    .find(|c| c.uuid == characteristic_id)
                    .ok_or(Error::NoSuchCharacteristic)?;
                self.peripheral.unsubscribe(&characteristic).await?;
                self.subscribers_count.remove(&characteristic_id);
            }
        }

        Ok(())
    }
}

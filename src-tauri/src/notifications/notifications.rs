use std::pin::Pin;

use btleplug::api::ValueNotification;
use futures_util::{
    stream::{AbortHandle, Abortable},
    Stream, StreamExt,
};

use crate::{connected_device::DeviceId, events::Events};

pub type CharacteristicValue = Vec<u8>;

pub struct Notifications {
    device_id: DeviceId,
    notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
    events: Events,
}

impl Notifications {
    fn new(
        device_id: DeviceId,
        notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
        events: Events,
    ) -> Self {
        Self {
            device_id,
            notification_stream,
            events,
        }
    }

    pub fn start(
        device_id: DeviceId,
        notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
        events: Events,
    ) -> AbortHandle {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        tokio::spawn(async move {
            Abortable::new(
                async move {
                    let mut notifications = Self::new(device_id, notification_stream, events);
                    notifications.run().await;
                },
                abort_registration,
            )
            .await
        });

        abort_handle
    }

    pub async fn run(&mut self) {
        while let Some(notification) = &self.notification_stream.next().await {
            self.events
                .on_notification(
                    self.device_id.clone(),
                    notification.uuid,
                    notification.value.clone(),
                )
                .await
                .expect("Failed to send notification");
        }
    }
}

use std::pin::Pin;

use btleplug::api::ValueNotification;
use futures_util::{
    stream::{AbortHandle, Abortable},
    Stream, StreamExt,
};
use tauri::async_runtime::Sender;

use crate::connected_device::DeviceId;

pub type NotificationListener = Sender<ValueNotification>;

/// Listens to notifications from all characteristics of a single Bluetooth device
pub struct Notifications {
    notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
    listener: NotificationListener,
}

impl Notifications {
    fn new(
        notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
        listener: NotificationListener,
    ) -> Self {
        Self {
            notification_stream,
            listener,
        }
    }

    pub fn start(
        device_id: DeviceId,
        notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
        listener: NotificationListener,
    ) -> AbortHandle {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        tokio::spawn(async move {
            Abortable::new(
                async move {
                    let mut notifications = Self::new(notification_stream, listener);
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
            self.listener.send(notification.clone()).await;
        }
    }
}

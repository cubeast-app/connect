use std::pin::Pin;

use btleplug::{
    api::{Peripheral as _, ValueNotification},
    platform::Peripheral,
    Error,
};
use futures_util::{Stream, StreamExt as _};

pub type NotificationStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

pub(super) async fn notification_stream(
    peripheral: &Peripheral,
    characteristic_id: uuid::Uuid,
) -> Result<NotificationStream, Error> {
    let notifications = peripheral.notifications().await?;
    let notifications = notifications.filter(move |notification| {
        let uuid = notification.uuid;
        async move { uuid == characteristic_id }
    });

    Ok(Box::pin(notifications))
}

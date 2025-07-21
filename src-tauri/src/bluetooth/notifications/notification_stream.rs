use std::pin::Pin;

use btleplug::{api::Peripheral as _, platform::Peripheral, Error};
use futures_util::{Stream, StreamExt as _};

use crate::bluetooth::{characteristic_value::CharacteristicValue, timestamp::timestamp};

pub type NotificationStream = Pin<Box<dyn Stream<Item = CharacteristicValue> + Send>>;

pub(super) async fn notification_stream(
    peripheral: &Peripheral,
    characteristic_id: uuid::Uuid,
) -> Result<NotificationStream, Error> {
    let notifications = peripheral.notifications().await?;
    let notifications = notifications
        .filter(move |notification| {
            let uuid = notification.uuid;
            async move { uuid == characteristic_id }
        })
        .map(|notification| CharacteristicValue {
            timestamp: timestamp(),
            value: notification.value,
        });

    Ok(Box::pin(notifications))
}

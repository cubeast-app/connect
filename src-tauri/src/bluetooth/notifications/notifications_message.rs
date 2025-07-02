use btleplug::Error;
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

use super::notification_stream::NotificationStream;

pub(crate) enum NotificationsMessage {
    Subscribe(Uuid, Sender<Result<NotificationStream, Error>>),
    Unsubscribe(Uuid, Sender<Result<(), Error>>),
    Stop,
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::connected_device::DeviceId;

#[derive(Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum Request {
    Authenticate,
    StartDiscovery,
    StopDiscovery,
    Connect {
        id: DeviceId,
    },
    Disconnect {
        id: DeviceId,
    },
    ReadCharacteristic {
        device_id: DeviceId,
        characteristic_id: Uuid,
    },
    WriteCharacteristic {
        device_id: DeviceId,
        characteristic_id: Uuid,
        value: Vec<u8>,
    },
    SubscribeCharacteristic {
        device_id: DeviceId,
        characteristic_id: Uuid,
    },
    UnsubscribeCharacteristic {
        device_id: DeviceId,
        characteristic_id: Uuid,
    },
    Version,
}

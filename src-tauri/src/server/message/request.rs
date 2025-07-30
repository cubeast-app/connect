use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Request {
    StartDiscovery,
    StopDiscovery,
    Connect {
        device_id: String,
    },
    Disconnect {
        device_id: String,
    },
    ReadCharacteristic {
        device_id: String,
        characteristic_id: Uuid,
    },
    WriteCharacteristic {
        device_id: String,
        characteristic_id: Uuid,
        value: Vec<u8>,
    },
    SubscribeToCharacteristic {
        device_id: String,
        characteristic_id: Uuid,
    },
    UnsubscribeFromCharacteristic {
        device_id: String,
        characteristic_id: Uuid,
    },
    Status,
}

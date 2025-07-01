use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Request {
    StartDiscovery,
    StopDiscovery,
    Connect { name: String },
    Disconnect { name: String },
    /*
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
    */
    Version,
}

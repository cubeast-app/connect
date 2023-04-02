use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum Request {
    Authenticate,
    StartDiscovery,
    StopDiscovery,
    Connect { id: String },
    Disconnect { id: String },
    /*
    ReadCharacteristic { id: String },
    WriteCharacteristic { id: String, value: Vec<u8> },
    SubscribeCharacteristic { id: String },
    UnsubscribeCharacteristic { id: String },
    */
    Version,
}

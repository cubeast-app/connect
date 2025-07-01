use serde::{Deserialize, Serialize};

use crate::bluetooth::device_data::DeviceData;

#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Response {
    Ok,
    Error { error: String },
    Value { value: Vec<u8> },
    Version { version: u16 },
    Connected { device: DeviceData },
}

impl Response {
    pub(crate) fn error(string: &str) -> Self {
        Self::Error {
            error: String::from(string),
        }
    }
}

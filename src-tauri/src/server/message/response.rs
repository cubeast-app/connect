use serde::{Deserialize, Serialize};

use crate::{bluetooth::device_data::DeviceData, app_status::Status};

#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Response {
    Ok,
    Error { error: String },
    Value { timestamp: u64, value: Vec<u8> },
    Status { status: Status },
    Connected { device: DeviceData },
}

impl Response {
    pub(crate) fn error(string: &str) -> Self {
        Self::Error {
            error: String::from(string),
        }
    }
}

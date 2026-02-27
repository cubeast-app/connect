use serde::{Deserialize, Serialize};

use crate::{
    app_status::Status,
    bluetooth::{
        device_data::DeviceData,
        error::{AppError, ErrorCategory, ErrorCode},
    },
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Response {
    Ok,
    Error {
        category: ErrorCategory,
        code: ErrorCode,
    },
    Value {
        timestamp: u64,
        value: Vec<u8>,
    },
    Status {
        status: Status,
    },
    Connected {
        device: DeviceData,
    },
}

impl From<AppError> for Response {
    fn from(err: AppError) -> Self {
        Self::Error {
            category: err.category,
            code: err.code,
        }
    }
}

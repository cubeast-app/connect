use serde::{Deserialize, Serialize};

pub mod broadcast;
pub mod request;
pub mod response;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Message {
    Request {
        id: String,
        request: request::Request,
    },
    Response {
        id: String,
        response: response::Response,
    },
    Broadcast {
        broadcast: broadcast::Broadcast,
    },
    Error {
        message: String,
    },
}

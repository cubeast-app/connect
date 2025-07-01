use serde::{Serialize, Deserialize};

pub mod request;
pub mod response;
pub mod broadcast;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Message {
  Request { id: String, request: request::Request },
  Response { id: String, response: response::Response },
  Broadcast { broadcast: broadcast::Broadcast },
  Error { message: String },
}

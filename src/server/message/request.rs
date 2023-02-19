use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "kebab-case")]
pub enum Request {
    Authenticate,
    StartDiscovery,
    StopDiscovery,
    Connect { id: String },
    Disconnect { id: String },
    Version,
}

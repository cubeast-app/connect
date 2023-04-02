use std::collections::HashMap;

use uuid::Uuid;

use crate::{clients::Client, server::message::broadcast::Broadcast};

#[derive(Debug)]
pub struct BroadcastCommand {
    pub clients: HashMap<Uuid, Client>,
    pub broadcast: Broadcast,
}

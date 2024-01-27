use crate::{clients::WebSocket, server::message::broadcast::Broadcast};

#[derive(Debug)]
pub struct BroadcastCommand {
    pub clients: Vec<WebSocket>,
    pub broadcast: Broadcast,
}

use crate::{clients::Client, server::message::broadcast::Broadcast};

#[derive(Debug)]
pub struct BroadcastCommand {
    pub clients: Vec<Client>,
    pub broadcast: Broadcast,
}

use std::{collections::HashMap, sync::Arc};

use futures_util::stream::SplitSink;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage, WebSocketStream};
use uuid::Uuid;

use crate::connected_device::ClientId;

pub type Client = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>>>;

pub struct Clients<ClientType: Clone> {
    internal: HashMap<ClientId, ClientType>,
}

impl<ClientType: Clone> Clients<ClientType> {
    pub fn new() -> Self {
        Self {
            internal: HashMap::new(),
        }
    }

    pub fn add(&mut self, client: ClientType) -> Uuid {
        let client_id = Uuid::new_v4();
        self.internal.insert(client_id, client);

        client_id
    }

    pub fn remove(&mut self, client_id: &ClientId) {
        self.internal.remove(client_id);
    }

    pub fn len(&self) -> usize {
        self.internal.len()
    }

    pub fn client_ids(&self) -> Vec<ClientId> {
        self.internal.keys().cloned().collect()
    }

    pub fn get_by_id(&self, id: &ClientId) -> Option<ClientType> {
        self.internal.get(id).cloned()
    }
}

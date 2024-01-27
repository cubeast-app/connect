use std::{collections::HashMap, sync::Arc};

use futures_util::stream::SplitSink;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage, WebSocketStream};
use uuid::Uuid;

use crate::connected_device::WebsocketClientId;

pub type WebSocket = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>>>;

pub struct Clients<ClientType> {
    internal: HashMap<WebsocketClientId, ClientType>,
}

impl<ClientType> Clients<ClientType> {
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

    pub fn remove(&mut self, client_id: &WebsocketClientId) {
        self.internal.remove(client_id);
    }

    pub fn len(&self) -> usize {
        self.internal.len()
    }

    pub fn client_ids(&self) -> Vec<WebsocketClientId> {
        self.internal.keys().cloned().collect()
    }

    pub fn get_by_id(&self, id: &WebsocketClientId) -> Option<&ClientType> {
        self.internal.get(id)
    }

    pub fn get_mut_by_id(&mut self, id: &WebsocketClientId) -> Option<&mut ClientType> {
        self.internal.get_mut(id)
    }
}

use std::{collections::HashMap, sync::Arc};

use futures_util::stream::SplitSink;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{self, tungstenite::Message as TungsteniteMessage, WebSocketStream};
use uuid::Uuid;

pub type Client = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>>>;

pub struct Clients {
    internal: HashMap<Uuid, Client>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            internal: HashMap::new(),
        }
    }

    pub fn add(&mut self, client: Client) -> Uuid {
        let client_id = Uuid::new_v4();
        self.internal.insert(client_id, client);

        client_id
    }

    pub fn remove(&mut self, client_id: Uuid) {
        self.internal.remove(&client_id);
    }

    pub fn len(&self) -> usize {
        self.internal.len()
    }

    pub fn get_by_ids(&self, client_ids: &[Uuid]) -> HashMap<Uuid, Client> {
        client_ids
            .iter()
            .filter_map(|client_id| {
                self.internal
                    .get(client_id)
                    .map(|client| (client_id.clone(), client.clone()))
            })
            .collect()
    }
}

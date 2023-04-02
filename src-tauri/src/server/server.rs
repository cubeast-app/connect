use std::{env, sync::Arc};

use crate::{controller::Controller, server::server_connection::ServerConnection};
use futures_util::StreamExt;
use log::info;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_tungstenite;

pub struct Server {
    controller: Controller,
}

impl Server {
    pub async fn start(controller: Controller) {
        tokio::spawn(async move {
            Self::new(controller).run().await;
        });
    }

    fn new(controller: Controller) -> Self {
        Self { controller }
    }

    async fn run(&self) {
        let addr = env::args()
            .nth(1)
            .unwrap_or_else(|| "127.0.0.1:17430".to_string());
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        info!("Listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            let (write, read) = ws_stream.split();

            let write = Arc::new(Mutex::new(write));

            let client_id = self.controller.add_client(write.clone()).await;

            ServerConnection::start(read, write, self.controller.clone(), client_id);
        }
    }
}

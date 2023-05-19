use std::{env, sync::Arc};

use crate::{controller::Controller, server::server_connection::ServerConnection};
use futures_util::StreamExt;
use http::{Response as HttpResponse, Uri};
use log::info;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_tungstenite::{
    self,
    tungstenite::handshake::{
        client::Request,
        server::{ErrorResponse, Response},
    },
};

pub const ALLOWED_HOSTS: [&str; 4] = [
    "localhost",
    "127.0.0.1",
    "app.cubeast.com",
    "app.staging.cubeast.com",
];

pub struct Server;

impl Server {
    pub fn start(controller: Controller) {
        tokio::spawn(async move {
            Self::run(controller).await;
        });
    }

    async fn run(controller: Controller) {
        let listener = create_tcp_listener().await;

        info!(
            "Listening on: {}",
            listener.local_addr().expect("Couldn't get local address")
        );

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_hdr_async(
                stream,
                |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
                    let origin = request.headers().get("origin");

                    if let Some(origin) = origin {
                        let uri = origin.to_str().unwrap().parse::<Uri>().unwrap();
                        let host = uri.host();

                        if let Some(host) = host {
                            if ALLOWED_HOSTS.contains(&host) {
                                return Ok(response);
                            }
                        }
                    }

                    info!("Rejected connection from {:?}", origin);
                    Err(HttpResponse::builder().status(403).body(None).unwrap())
                },
            )
            .await;

            if let Ok(ws_stream) = ws_stream {
                let (write, read) = ws_stream.split();
                let write = Arc::new(Mutex::new(write));
                let client_id = controller.add_client(write.clone()).await;

                ServerConnection::start(read, write, controller.clone(), client_id);
            }
        }
    }
}

async fn create_tcp_listener() -> TcpListener {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:17430".to_string());
    let try_socket = TcpListener::bind(&addr).await;
    try_socket.expect("Failed to bind")
}

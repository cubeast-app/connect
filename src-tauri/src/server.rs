use futures_util::StreamExt;
use http::{Response as HttpResponse, Uri};
use tokio::net::TcpListener;
use tokio_tungstenite::{
    self,
    tungstenite::handshake::{
        client::Request,
        server::{ErrorResponse, Response},
    },
};
use tracing::{info, trace, warn};

use crate::{app_status::AppStatus, bluetooth::Bluetooth, server::connection::Connection};

mod connection;
mod message;

pub const ALLOWED_HOSTS: [&str; 3] = [
    "app.cubeast.com",
    "app.staging.cubeast.com",
    "app.beta.cubeast.com",
];

pub struct Server;

impl Server {
    pub(crate) fn start(
        bluetooth: Bluetooth,
        app_status: AppStatus,
        bind_addr: String,
        allow_any_origin: bool,
    ) {
        let bluetooth_clone = bluetooth.clone();
        let app_status_clone = app_status.clone();

        tokio::spawn(async move {
            Self::run(
                bluetooth_clone,
                app_status_clone,
                bind_addr,
                allow_any_origin,
            )
            .await;
        });
    }

    async fn run(
        bluetooth: Bluetooth,
        app_status: AppStatus,
        bind_addr: String,
        allow_any_origin: bool,
    ) {
        let listener = create_tcp_listener(&bind_addr).await;

        info!(
            "Listening on: {}",
            listener.local_addr().expect("Couldn't get local address")
        );

        if allow_any_origin {
            warn!("--allow-any-origin flag enabled - any origin accepted (INSECURE - for development only)");
        }

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_hdr_async(
                stream,
                |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
                    let origin = request.headers().get("origin");

                    // If --allow-any-origin flag is set, we allow any origin
                    if allow_any_origin {
                        trace!("--allow-any-origin flag enabled, allowing any origin");
                        return Ok(response);
                    }

                    if let Some(origin) = origin {
                        trace!("Origin header: {origin:?}");
                        let uri = origin.to_str().unwrap_or_default().parse::<Uri>();

                        if let Ok(uri) = uri {
                            let host = uri.host();

                            if let Some(host) = host {
                                trace!("Received connection from host: {host}");

                                // Check production allowed hosts
                                if ALLOWED_HOSTS.contains(&host) {
                                    return Ok(response);
                                }
                            }
                        }
                    }

                    info!("Rejected connection from {origin:?}");
                    Err(HttpResponse::builder().status(403).body(None).unwrap())
                },
            )
            .await;

            if let Ok(ws_stream) = ws_stream {
                let (write, read) = ws_stream.split();
                Connection::start(bluetooth.clone(), app_status.clone(), read, write);
            }
        }
    }
}

async fn create_tcp_listener(bind_addr: &str) -> TcpListener {
    let try_socket = TcpListener::bind(bind_addr).await;
    try_socket.expect("Failed to bind")
}

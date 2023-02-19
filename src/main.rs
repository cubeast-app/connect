mod adapter;
mod app_state;
mod bluetooth_error;
mod broadcaster;
mod clients;
mod controller;
mod discovery;
mod server;
mod version;

extern crate pretty_env_logger;

use std::sync::Arc;

use crate::server::server::Server;
use app_state::AppState;
use broadcaster::{broadcast_command::BroadcastCommand, broadcaster::Broadcaster};
use controller::Controller;
use discovery::{discovery::Discovery, discovery_command::DiscoveryCommand};
use tokio::sync::{mpsc, Mutex};

const CHANNEL_CAPACITY: usize = 64;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (broadcaster_tx, broadcaster_rx) = mpsc::channel::<BroadcastCommand>(CHANNEL_CAPACITY);
    let (discovery_tx, discovery_rx) = mpsc::channel::<DiscoveryCommand>(CHANNEL_CAPACITY);

    let app_state = Arc::new(Mutex::new(AppState::new()));

    let controller = Controller::new(app_state, discovery_tx, broadcaster_tx);

    Broadcaster::start(broadcaster_rx).await;
    Discovery::start(controller.clone(), discovery_rx).await;
    Server::start(controller).await;
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
use adapter::bluetooth_adapter;
use app_state::AppState;
use broadcaster::{broadcast_command::BroadcastCommand, broadcaster::Broadcaster};
use controller::Controller;
use discovery::{discovery::Discovery, discovery_command::DiscoveryCommand};
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tokio::sync::{mpsc, Mutex};

const CHANNEL_CAPACITY: usize = 64;

#[tauri::command]
async fn status(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<usize, String> {
    bluetooth_adapter()
        .await
        .map_err(|error| error.to_string())?;

    let state = state.lock().await;
    Ok(state.clients.len())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (broadcaster_tx, broadcaster_rx) = mpsc::channel::<BroadcastCommand>(CHANNEL_CAPACITY);
    let (discovery_tx, discovery_rx) = mpsc::channel::<DiscoveryCommand>(CHANNEL_CAPACITY);

    let app_state = Arc::new(Mutex::new(AppState::new()));

    let controller = Controller::new(app_state.clone(), discovery_tx, broadcaster_tx);

    Broadcaster::start(broadcaster_rx).await;
    Discovery::start(controller.clone(), discovery_rx).await;
    Server::start(controller).await;

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("open", "Open"))
        .add_item(CustomMenuItem::new("exit", "Exit"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .manage(app_state)
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![status])
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "exit" => {
                    app.exit(0);
                }
                "open" => {
                    let window = app.get_window("main").unwrap();
                    window.maximize().unwrap();
                }
                _ => {
                    eprintln!("Unknown menu item: {}", id);
                }
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running Cubeast Connect");
}

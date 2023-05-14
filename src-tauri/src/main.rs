// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod bluetooth;
mod bluetooth_error;
mod bluetooth_state;
mod broadcaster;
mod clients;
mod connected_device;
mod controller;
mod disconnections;
mod discovered_device;
mod discovery;
mod events;
mod notifications;
mod server;
mod version;

extern crate pretty_env_logger;

use std::sync::Arc;

use crate::server::server::Server;
use bluetooth_state::BluetoothState;
use broadcaster::broadcaster::Broadcaster;
use controller::Controller;
use events::Events;
use tauri::async_runtime::Mutex;
use tauri::{
    AppHandle, CustomMenuItem, GlobalWindowEvent, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, Wry,
};

pub const CHANNEL_CAPACITY: usize = 64;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (broadcaster_tx, broadcaster_rx) = tokio::sync::mpsc::channel(CHANNEL_CAPACITY);
    let bluetooth_state = BluetoothState::new();
    let bluetooth_state = Arc::new(Mutex::new(bluetooth_state));
    let events = Events::new(bluetooth_state.clone(), broadcaster_tx.clone());

    let mut locked_state = bluetooth_state.lock().await;
    locked_state.set_events(events);
    locked_state
        .start()
        .await
        .expect("Bluetooth startup failed");
    drop(locked_state);

    let controller = Controller::new(bluetooth_state);
    Broadcaster::start(broadcaster_rx);
    Server::start(controller.clone());

    build_tauri(controller)
        .run(tauri::generate_context!())
        .expect("error while running Cubeast Connect");
}

#[tauri::command]
async fn status(controller: tauri::State<'_, Controller>) -> Result<usize, String> {
    Ok(controller.client_count().await)
}

fn build_tauri(controller: Controller) -> tauri::Builder<Wry> {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("open", "Open"))
        .add_item(CustomMenuItem::new("exit", "Exit"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .manage(controller)
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![status])
        .on_system_tray_event(handle_system_tray_event)
        .on_window_event(handle_window_event)
}

fn handle_window_event(event: GlobalWindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
        event.window().hide().unwrap();
        api.prevent_close();
    }
}

fn handle_system_tray_event(app: &AppHandle<Wry>, event: SystemTrayEvent) {
    if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
            "exit" => {
                app.exit(0);
            }
            "open" => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.unminimize().unwrap();
                window.set_focus().unwrap();
            }
            _ => {
                eprintln!("Unknown menu item: {}", id);
            }
        }
    }
}

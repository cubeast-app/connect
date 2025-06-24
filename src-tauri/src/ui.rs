use std::thread;

use futures_util::StreamExt;
use log::{error, trace};
use tauri::{
    async_runtime::block_on, AppHandle, CustomMenuItem, GlobalWindowEvent, Manager as _, State,
    SystemTray, SystemTrayEvent, SystemTrayMenu, Wry,
};
use tauri_plugin_autostart::MacosLauncher;

use crate::bluetooth::Bluetooth;

struct Context {
    bluetooth: Bluetooth,
}

#[tauri::command]
async fn start_discovery(
    app_handle: tauri::AppHandle,
    context: State<'_, Context>,
) -> Result<(), String> {
    trace!("Starting discovery from UI");
    let discovered_devices = context
        .bluetooth
        .subscribe_to_discovery()
        .await
        .map_err(|err| err.to_string())
        .expect("Failed to start discovery");

    thread::spawn(move || {
        let mut devices_stream = discovered_devices.fuse();
        let app_handle = app_handle.clone();

        while let Some(devices) = block_on(devices_stream.next()) {
            trace!("Discovered devices: {:?}", devices);
            app_handle.emit_all("discovery", devices).unwrap();
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_discovery(context: State<'_, Context>) -> Result<(), String> {
    trace!("Stopping discovery from UI");
    context.bluetooth.unsubscribe_from_discovery().await;

    Ok(())
}

pub fn build_tauri(bluetooth: Bluetooth) -> tauri::Builder<Wry> {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("open", "Open"))
        .add_item(CustomMenuItem::new("exit", "Exit"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(Context { bluetooth })
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![start_discovery, stop_discovery])
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
                error!("Unknown menu item: {}", id);
            }
        }
    }
}

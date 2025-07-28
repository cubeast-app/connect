use std::{thread, time::Duration};

use futures_util::StreamExt;
use log::{error, info, trace};
use tauri::{
    async_runtime::block_on,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter as _, Manager, State, WindowEvent, Wry,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_opener::open_url;

use crate::bluetooth::{device_data::DeviceData, Bluetooth};

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
            app_handle.emit("discovery", devices).unwrap();
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

#[tauri::command]
async fn device_details(
    context: State<'_, Context>,
    device_id: String,
) -> Result<DeviceData, String> {
    info!("Fetching details for device: {}", device_id);
    let connection_future = context.bluetooth.connect(&device_id);

    let timeout = Duration::from_secs(15);
    let device_data = tokio::time::timeout(timeout, connection_future)
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())?;

    context
        .bluetooth
        .disconnect(&device_id)
        .await
        .map_err(|err| err.to_string())?;

    info!("Device data: {:?}", device_data);

    Ok(device_data)
}

pub fn build_tauri(bluetooth: Bluetooth) -> tauri::Builder<Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Context { bluetooth })
        .invoke_handler(tauri::generate_handler![
            start_discovery,
            stop_discovery,
            device_details
        ])
        .setup(|app| {
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(
                    &tauri::menu::MenuBuilder::new(app)
                        .text("open", "Open")
                        .text("cubeast", "Go to Cubeast")
                        .text("exit", "Exit")
                        .build()?,
                )
                .on_menu_event(handle_menu_event)
                .on_tray_icon_event(handle_tray_icon_event)
                .build(app)?;

            Ok(())
        })
        .on_window_event(handle_window_event)
}

fn handle_window_event(window: &tauri::Window<Wry>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        window.hide().unwrap();
        api.prevent_close();
    }
}

fn handle_menu_event(app: &AppHandle<Wry>, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "exit" => {
            app.exit(0);
        }
        "open" => {
            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
            window.unminimize().unwrap();
            window.set_focus().unwrap();
        }
        "cubeast" => {
            if let Err(err) = open_url("https://app.cubeast.com", None::<String>) {
                error!("Failed to open Cubeast: {}", err);
            }
        }
        _ => {
            error!("Unknown menu item: {:?}", event.id());
        }
    }
}

fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon<Wry>, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let app = tray.app_handle();
        let window = app.get_webview_window("main").unwrap();
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
    }
}

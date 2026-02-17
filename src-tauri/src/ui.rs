use std::{thread, time::Duration};

use futures_util::StreamExt;
use tauri::{
    async_runtime::block_on,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter as _, Manager, State, WindowEvent, Wry,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_cli::CliExt as _;
use tauri_plugin_opener::open_url;
use tauri_plugin_updater::UpdaterExt as _;
use tracing::{error, info, trace};

use crate::app_status::{AppStatus, Status};
use crate::bluetooth::{device_data::DeviceData, Bluetooth};
use crate::server::Server;

struct Context {
    bluetooth: Bluetooth,
    status: AppStatus,
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
            if let Err(e) = app_handle.emit("discovery", devices) {
                error!("Failed to emit discovery event: {e}");
            }
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
    info!("Fetching details for device: {device_id}");
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

    info!("Device data: {device_data:?}");

    Ok(device_data)
}

#[tauri::command]
async fn app_status(context: State<'_, Context>) -> Result<Status, String> {
    Ok(context.status.get().await)
}

pub fn build_tauri(bluetooth: Bluetooth, status: AppStatus) -> tauri::Builder<Wry> {
    let status_for_tauri = status.clone();
    let bluetooth_for_setup = bluetooth.clone();
    let status_for_setup = status.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Context {
            bluetooth,
            status: status.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            start_discovery,
            stop_discovery,
            device_details,
            app_status
        ])
        .setup(move |app| {
            // Parse CLI arguments and start WebSocket server
            if let Ok(matches) = app.cli().matches() {
                let bind_addr = matches
                    .args
                    .get("bind")
                    .and_then(|value| value.value.as_str())
                    .unwrap_or("127.0.0.1:17430")
                    .to_string();

                let allow_any_origin = matches
                    .args
                    .get("allow-any-origin")
                    .and_then(|value| value.value.as_bool())
                    .unwrap_or(false);

                Server::start(
                    bluetooth_for_setup.clone(),
                    status_for_setup.clone(),
                    bind_addr,
                    allow_any_origin,
                );
            }

            #[cfg(desktop)]
            let _ = app
                .handle()
                .plugin(tauri_plugin_updater::Builder::new().build());

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(
                    &tauri::menu::MenuBuilder::new(app)
                        .text("open", "Open")
                        .text("update", "Check for updates")
                        .text("cubeast", "Go to Cubeast")
                        .text("exit", "Exit")
                        .build()?,
                )
                .on_menu_event(handle_menu_event)
                .on_tray_icon_event(handle_tray_icon_event)
                .build(app)?;

            let handle = app.handle().clone();
            let status_clone = status_for_tauri.clone();

            tokio::spawn(async move {
                let mut status_rx = status_clone.subscribe();
                loop {
                    if let Ok(status) = status_rx.recv().await {
                        if let Err(e) = handle.emit("app_status_changed", status) {
                            error!("Failed to emit status event: {e}");
                        }
                    }
                }
            });

            let handle = app.handle().clone();
            let status_clone = status_for_tauri.clone();

            tauri::async_runtime::spawn(async move {
                update(handle, status_clone).await.unwrap();
            });

            if let Ok(matches) = app.cli().matches() {
                let background = matches
                    .args
                    .get("background")
                    .and_then(|value| value.value.as_bool())
                    .unwrap_or(false);

                let window = app.get_webview_window("main").unwrap();
                if background {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
            }

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
            window.center().unwrap();
            window.show().unwrap();
            window.unminimize().unwrap();
            window.set_focus().unwrap();
        }
        "cubeast" => {
            if let Err(err) = open_url("https://app.cubeast.com", None::<String>) {
                error!("Failed to open Cubeast: {err}");
            }
        }
        "update" => {
            let handle = app.clone();
            let status_clone = app.state::<Context>().status.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(err) = update(handle, status_clone).await {
                    error!("Failed to check for updates: {err}");
                }
            });
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

async fn update(app: tauri::AppHandle, app_status: AppStatus) -> tauri_plugin_updater::Result<()> {
    app_status.update(Status::CheckingForUpdates).await;

    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;
        let mut progress = 0;

        info!("Update available: {}", update.version);

        update
            .download_and_install(
                move |chunk_length, content_length| {
                    downloaded += chunk_length;

                    let new_progress = if let Some(total) = content_length {
                        ((downloaded as f64 / total as f64) * 100.0) as u8
                    } else {
                        0
                    };

                    if new_progress != progress {
                        progress = new_progress;
                        let app_status_clone = app_status.clone();
                        tokio::spawn(async move {
                            app_status_clone
                                .update(Status::DownloadingUpdate {
                                    progress: new_progress,
                                })
                                .await;
                        });
                    }
                },
                || {
                    info!("Download finished");
                },
            )
            .await?;

        info!("Update installed");
        app.restart();
    } else {
        // No update available, set status to running
        app_status
            .update(Status::Running {
                version: env!("CARGO_PKG_VERSION").to_string(),
            })
            .await;
    }

    Ok(())
}

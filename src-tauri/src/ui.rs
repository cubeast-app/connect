use log::{error, info};
use tauri::{
    AppHandle, CustomMenuItem, GlobalWindowEvent, Manager as _, State, SystemTray, SystemTrayEvent,
    SystemTrayMenu, Wry,
};
use tauri_plugin_autostart::MacosLauncher;

use crate::main_controller::MainController;

struct Context {
    main_controller: MainController,
}

#[tauri::command]
async fn start_discovery(
    app_handle: tauri::AppHandle,
    context: State<'_, Context>,
) -> Result<(), String> {
    info!("Starting discovery from UI");
    context
        .main_controller
        .start_discovery(app_handle)
        .await
        .map_err(|_| "Failed to start discovery".to_string())
}

#[tauri::command]
async fn stop_discovery(context: State<'_, Context>) -> Result<(), String> {
    info!("Stopping discovery from UI");
    context
        .main_controller
        .stop_discovery()
        .await
        .map_err(|_| "Failed to start discovery".to_string())
}

pub fn build_tauri(main_controller: MainController) -> tauri::Builder<Wry> {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("open", "Open"))
        .add_item(CustomMenuItem::new("exit", "Exit"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(Context { main_controller })
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

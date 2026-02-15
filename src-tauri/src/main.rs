// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(
    clippy::use_self,
    clippy::cognitive_complexity,
    clippy::cloned_instead_of_copied,
    clippy::derive_partial_eq_without_eq,
    clippy::equatable_if_let,
    clippy::explicit_into_iter_loop,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::match_same_arms,
    clippy::needless_for_each,
    clippy::todo
)]

mod app_status;
mod bluetooth;
mod server;
mod ui;

use bluetooth::Bluetooth;

use crate::app_status::AppStatus;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    let adapter = bluetooth::adapter()
        .await
        .expect("Failed to connect to the Bluetooth adapter");
    let bluetooth = Bluetooth::start(adapter);
    let app_status = AppStatus::new();

    ui::build_tauri(bluetooth, app_status)
        .run(tauri::generate_context!())
        .expect("error while running Cubeast Connect");
}

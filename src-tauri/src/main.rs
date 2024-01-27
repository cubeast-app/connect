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

mod bluetooth;
mod device_id;
mod main_controller;
mod server;
mod ui;
mod version;

use bluetooth::Bluetooth;
use server::Server;
use ui::build_tauri;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bluetooth = Bluetooth::new().await;

    Server::start(bluetooth.clone());

    let main_controller = main_controller::MainController::start(bluetooth);

    build_tauri(main_controller)
        .run(tauri::generate_context!())
        .expect("error while running Cubeast Connect");
}

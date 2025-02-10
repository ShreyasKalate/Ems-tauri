#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::{system::get_ram_usage, system::get_running_apps, apps::get_installed_apps, browser::get_browser_history};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_ram_usage,
            get_running_apps,
            get_installed_apps,
            get_browser_history
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

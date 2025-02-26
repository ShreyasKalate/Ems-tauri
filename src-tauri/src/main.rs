#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::system::{get_ram_usage, track_ram_usage};
use commands::{
    installed_apps::get_installed_apps,
    browser::get_browser_history,
    visible_apps::get_visible_apps,
    running_apps::get_running_apps,
};

fn main() {
    track_ram_usage();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_visible_apps,
            get_running_apps,
            get_ram_usage,
            get_installed_apps,
            get_browser_history
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

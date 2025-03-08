#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::{
    system::{get_ram_usage, track_ram_usage},
    installed_apps::get_installed_apps,
    browser::get_browser_history,
    visible_apps::get_visible_apps,
    running_apps::get_running_apps,
    capture_screen::{get_capture_screen, start_screenshot_scheduler},
    usb_devices::list_usb_devices,
    usb_monitor::monitor_usb_file_transfers,
};
use tauri::Manager;
use tokio::runtime::Runtime;

fn main() {
    track_ram_usage();
    monitor_usb_file_transfers();

    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    runtime.spawn(async {
        start_screenshot_scheduler().await;
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_visible_apps,
            get_running_apps,
            get_ram_usage,
            get_installed_apps,
            get_browser_history,
            get_capture_screen,
            list_usb_devices,
        ])
        .setup(|_app| {
            println!("Tauri app is running...");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::{
    system::{get_ram_usage, track_ram_usage},
    installed_apps::store_installed_apps_to_db,
    browser::get_browser_history,
    visible_apps::track_visible_apps,
    running_apps::get_running_apps,
    capture_screen::{get_capture_screen, start_screenshot_scheduler},
    usb_devices::{start_usb_monitor, init_usb_table}, // ✅ USB functions
    usb_monitor::monitor_usb_file_transfers,
    afk_tracker::{start_afk_tracker, get_afk_status,init_afk_db},
};
use tokio::runtime::Runtime;
use std::thread;

fn main() {
    track_ram_usage();
    start_afk_tracker();
    track_visible_apps();
    
    thread::spawn(|| {
        store_installed_apps_to_db();
    });

    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    runtime.spawn(async {
        monitor_usb_file_transfers();
    });

    runtime.spawn(async {
        start_screenshot_scheduler().await;
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_afk_status,
            get_running_apps,
            get_ram_usage,
            get_browser_history,
            get_capture_screen,
            monitor_usb_file_transfers,
        ])
        .setup(|_app| {
            println!("Tauri app is running...");

            init_usb_table();
            init_afk_db(); // ✅ Create USB table if not exists
            start_usb_monitor(); // ✅ Start monitoring USB devices automatically

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

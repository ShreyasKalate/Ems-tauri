#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;

mod commands;

use commands::database::{create_tables, start_db_operations, clean_exit};
use commands::{
    system::track_ram_usage,
    installed_apps::store_installed_apps_to_db,
    browser::get_browser_history,
    visible_apps::track_visible_apps,
    running_apps::get_running_apps,
    capture_screen::{get_capture_screen, start_screenshot_scheduler},
    usb_devices::list_usb_devices,
    usb_monitor::monitor_usb_file_transfers,
    afk_tracker::{start_afk_tracker, get_afk_status},
};
use tokio::runtime::Runtime;

fn main() {
    // 🧠 Start RAM tracker (cache-only)
    thread::spawn(track_ram_usage);
    start_afk_tracker();
    track_visible_apps();
    store_installed_apps_to_db();

    // 🛠️ Initialize DB schema
    create_tables();

    // 🔁 Start background DB sync loop
    thread::spawn(start_db_operations);

    // 🧵 Start async services
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    runtime.spawn(async {
        monitor_usb_file_transfers();
    });

    runtime.spawn(async {
        start_screenshot_scheduler().await;
    });

    // Setup Ctrl+C for clean exit
    ctrlc::set_handler(|| {
        println!("🚪 Exiting...");
        clean_exit();
        std::process::exit(0);
    }).expect("❌ Failed to set Ctrl+C handler");

    // Start Tauri app
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_afk_status,
            get_running_apps,
            get_browser_history,
            get_capture_screen,
            list_usb_devices,
            monitor_usb_file_transfers,
        ])
        .setup(|_app| {
            println!("🚀 Tauri app is running...");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}

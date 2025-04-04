#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use once_cell::sync::Lazy;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};

mod commands;

use commands::{
    afk_tracker,
    browser,
    capture_screen,
    installed_apps,
    running_apps,
    system,
    usb_devices,
    usb_monitor,
    visible_apps,
};

// Shutdown flag
pub static SHUTDOWN_FLAG: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

// r2d2 connection pool
pub static DB_POOL: Lazy<Pool<SqliteConnectionManager>> = Lazy::new(|| {
    let manager = SqliteConnectionManager::file("ems_data.db");
    Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("❌ Failed to create SQLite connection pool")
});

pub fn get_conn() -> PooledConnection<SqliteConnectionManager> {
    DB_POOL
        .get()
        .expect("❌ Failed to get DB connection from pool")
}

fn main() {
    // Ctrl+C graceful shutdown
    ctrlc::set_handler(|| {
        println!("🛑 Ctrl+C detected. Shutting down...");
        SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
        // std::process::exit(0);   // Uncomment if thread is stuck, error is discovered
    })
    .expect("❌ Failed to set Ctrl+C handler");

    // Create all tables

    // afk_tracker::create_table();
    // browser::create_table();
    // capture_screen::create_table();
    installed_apps::create_table();
    // running_apps::create_table();
    system::create_table();
    // usb_monitor::create_table();
    // usb_devices::create_table();
    // visible_apps::create_table();

    // Installed apps thread (every 60s, flush every 1 mins)
    thread::spawn(|| {
        let mut tick = 0;
        loop {
            if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
                installed_apps::flush_cache();
                break;
            }

            let data = installed_apps::collect_info();
            installed_apps::push_to_cache(data);

            tick += 1;
            if tick % 1 == 0 {
                installed_apps::flush_cache();
            }

            thread::sleep(Duration::from_secs(60));
        }
    });

    // System thread (every 5s, flush every 15s)
    thread::spawn(|| {
        let mut tick = 0;
        loop {
            if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
                system::flush_cache();
                break;
            }

            let snap = system::collect_info();
            system::push_to_cache(snap);

            tick += 1;
            if tick % 3 == 0 {
                system::flush_cache();
            }

            thread::sleep(Duration::from_secs(5));
        }
    });

    // Visible apps thread (every 10s, flush every 60s)
    // thread::spawn(|| {
    //     let mut tick = 0;
    //     loop {
    //         if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
    //             visible_apps::flush_cache();
    //             break;
    //         }

    //         let data = visible_apps::collect_info();
    //         visible_apps::push_to_cache(data);

    //         tick += 1;
    //         if tick % 6 == 0 {
    //             visible_apps::flush_cache();
    //         }

    //         thread::sleep(Duration::from_secs(10));
    //     }
    // });

    // // Background modules with their own thread
    // thread::spawn(|| {
    //     capture_screen::collect_and_store(); // loops internally
    // });

    // thread::spawn(|| {
    //     usb_monitor::collect_and_store(); // loops internally
    // });

    // afk_tracker::collect_and_store(); // spawns its own loop internally

    // Tauri is optional, non-triggering
    // tauri::Builder::default()
    //     .invoke_handler(tauri::generate_handler![
    //         afk_tracker::get_afk_status,
    //         running_apps::get_running_apps,
    //         browser::get_browser_history,
    //         capture_screen::get_capture_screen,
    //         usb_devices::list_usb_devices,
    //         usb_monitor::monitor_usb_file_transfers,
    //     ])
    //     .setup(|_app| {
    //         println!("🚀 Tauri app is running...");
    //         Ok(())
    //     })
    //     .run(tauri::generate_context!())
    //     .expect("❌ Failed to run Tauri app");

    // Instead of running a GUI, just wait forever until shutdown
    loop {
        if SHUTDOWN_FLAG.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }

    // Final cleanup
    installed_apps::flush_cache();
    system::flush_cache();

    // visible_apps::flush_cache();
    // usb_monitor::flush_cache();

    println!("✅ Shutdown complete. Everything flushed.");
}

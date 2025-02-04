// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sysinfo::System;
use winreg::enums::*;
use winreg::RegKey;
use std::collections::HashSet;
use tauri::command;

#[command]
fn get_running_apps() -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    sys.processes()
        .iter()
        .map(|(pid, process)| format!("{} (PID: {})", process.name().to_string_lossy(), pid))
        .collect()
}

#[command]
fn get_installed_apps() -> Vec<String> {
    let mut apps: HashSet<String> = HashSet::new();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        .ok();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        .ok();

    if let Some(key) = hklm {
        for subkey_name in key.enum_keys().filter_map(Result::ok) {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
                    apps.insert(display_name);
                }
            }
        }
    }

    if let Some(key) = hkcu {
        for subkey_name in key.enum_keys().filter_map(Result::ok) {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
                    apps.insert(display_name);
                }
            }
        }
    }

    apps.into_iter().collect()
}

#[command]
fn get_ram_usage() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    format!(
        "Total RAM: {:.2} GB\nUsed RAM: {:.2} GB",
        total_memory,
        used_memory
    )
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_ram_usage, get_running_apps, get_installed_apps])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

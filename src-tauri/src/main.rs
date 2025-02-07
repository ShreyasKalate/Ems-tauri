#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use winreg::enums::*;
use winreg::RegKey;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use std::process::{Command, Stdio};
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

#[derive(Serialize, Deserialize, Debug)]
struct InstalledApp {
    identifying_number: String,
    install_date: String,
    install_location: String,
    name: String,
    vendor: String,
    version: String,
}

// Function to get identifying numbers using `wmic`
fn get_msi_installed_apps() -> Vec<(String, String)> {
    let output = Command::new("wmic")
        .args(["product", "get", "IdentifyingNumber,Name"])
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut apps = Vec::new();

        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() > 1 {
            for line in lines.iter().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let identifying_number = parts[0].to_string();
                    let name = parts[1..].join(" ");
                    apps.push((name, identifying_number));
                }
            }
        }
        return apps;
    }
    vec![]
}

#[command]
fn get_installed_apps() -> Vec<InstalledApp> {
    let mut apps: Vec<InstalledApp> = Vec::new();
    let msi_apps = get_msi_installed_apps();

    // Access Windows Registry
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        .ok();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        .ok();

    let mut extract_info = |key: &RegKey| {
        for subkey_name in key.enum_keys().filter_map(Result::ok) {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                let name = subkey.get_value::<String, _>("DisplayName").unwrap_or_default();
                if name.is_empty() {
                    continue;
                }
                
                let identifying_number = msi_apps
                    .iter()
                    .find(|(app_name, _)| app_name == &name)
                    .map(|(_, id)| id.clone())
                    .unwrap_or("N/A".to_string());

                let install_date = subkey.get_value::<String, _>("InstallDate").unwrap_or("N/A".to_string());
                let install_location = subkey.get_value::<String, _>("InstallLocation").unwrap_or("N/A".to_string());
                let vendor = subkey.get_value::<String, _>("Publisher").unwrap_or("Unknown".to_string());
                let version = subkey.get_value::<String, _>("DisplayVersion").unwrap_or("Unknown".to_string());

                apps.push(InstalledApp {
                    identifying_number,
                    install_date,
                    install_location,
                    name,
                    vendor,
                    version,
                });
            }
        }
    };

    if let Some(key) = hklm {
        extract_info(&key);
    }

    if let Some(key) = hkcu {
        extract_info(&key);
    }

    apps
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

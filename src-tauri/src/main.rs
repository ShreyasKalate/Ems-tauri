#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use winreg::enums::*;
use winreg::RegKey;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use std::process::Command;
use tauri::command;
use chrono::{Utc, NaiveDateTime};

#[derive(Serialize, Deserialize)]
struct RunningApp {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory_usage: u64,
    start_time: String,
}

#[command]
fn get_running_apps() -> Vec<RunningApp> {
    let mut sys = System::new_all();
    sys.refresh_all();

    sys.processes()
        .iter()
        .map(|(pid, process)| {
            let start_time = chrono::DateTime::from_timestamp(process.start_time() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or("Unknown".to_string());

            RunningApp {
                name: process.name().to_string_lossy().to_string(),
                pid: pid.as_u32(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                start_time,
            }
        })
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

fn format_date(date: &str) -> String {
    if date.len() == 8 {
        format!(
            "{}-{}-{}",
            &date[6..],  // Day
            &date[4..6], // Month
            &date[0..4]  // Year
        )
    } else {
        "N/A".to_string()
    }
}

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
fn get_installed_apps() -> (Vec<InstalledApp>, Vec<InstalledApp>) {
    let mut system_apps: Vec<InstalledApp> = Vec::new();
    let mut user_apps: Vec<InstalledApp> = Vec::new();
    let msi_apps = get_msi_installed_apps();

    // Extract information from registry keys
    let extract_info = |key: &RegKey, app_list: &mut Vec<InstalledApp>| {
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

                let install_date = subkey.get_value::<String, _>("InstallDate")
                    .map(|d| format_date(&d))
                    .unwrap_or("N/A".to_string());

                let install_location = subkey.get_value::<String, _>("InstallLocation").unwrap_or("N/A".to_string());
                let vendor = subkey.get_value::<String, _>("Publisher").unwrap_or("Unknown".to_string());
                let version = subkey.get_value::<String, _>("DisplayVersion").unwrap_or("Unknown".to_string());

                app_list.push(InstalledApp {
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

    // Retrieve system-wide installed apps from HKLM (HKEY_LOCAL_MACHINE)
    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
        extract_info(&hklm, &mut system_apps);
    }

    // Retrieve user-specific installed apps from HKCU (HKEY_CURRENT_USER)
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
        extract_info(&hkcu, &mut user_apps);
    }

    (system_apps, user_apps)
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

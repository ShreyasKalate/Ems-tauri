#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use winreg::enums::*;
use winreg::RegKey;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::process::Command;
use tauri::command;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::fs;
use std::env;
use std::collections::HashMap;
use chrono::{DateTime, Local, NaiveDateTime, Utc};

/// Get a list of Chrome profile directories.
fn get_chrome_profiles() -> Vec<PathBuf> {
    let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    let chrome_base_path = PathBuf::from(format!(
        "{}\\AppData\\Local\\Google\\Chrome\\User Data",
        user_profile
    ));

    let mut profiles = Vec::new();

    if let Ok(entries) = fs::read_dir(chrome_base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let history_db = path.join("History");
                if history_db.exists() {
                    profiles.push(path);
                }
            }
        }
    }

    profiles
}

/// Extracts display names for Chrome profiles from `Local State`.
fn get_profile_display_names() -> HashMap<String, String> {
    let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    let local_state_path = PathBuf::from(format!(
        "{}\\AppData\\Local\\Google\\Chrome\\User Data\\Local State",
        user_profile
    ));

    let mut profile_map = HashMap::new();

    if let Ok(data) = fs::read_to_string(local_state_path) {
        if let Ok(json) = serde_json::from_str::<Value>(&data) {
            if let Some(profiles) = json.get("profile") {
                if let Some(info_cache) = profiles.get("info_cache") {
                    if let Some(profiles_map) = info_cache.as_object() {
                        for (profile_key, profile_info) in profiles_map {
                            if let Some(display_name) = profile_info.get("name").and_then(Value::as_str) {
                                profile_map.insert(profile_key.clone(), display_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    profile_map
}

#[derive(Serialize, Deserialize, Debug)]
struct BrowserHistory {
    profile: String,
    profile_display_name: String,
    gmail: String,
    title: String,
    url: String,
    visit_time: String,
}

/// Extract Gmail ID from Chrome's `Preferences` JSON file.
fn get_gmail_for_profile(profile_path: &PathBuf) -> String {
    let preferences_path = profile_path.join("Preferences");

    if !preferences_path.exists() {
        return "Unknown".to_string();
    }

    if let Ok(data) = fs::read_to_string(&preferences_path) {
        if let Ok(json) = serde_json::from_str::<Value>(&data) {
            if let Some(accounts) = json.get("account_info").and_then(Value::as_array) {
                for account in accounts {
                    if let Some(email) = account.get("email").and_then(Value::as_str) {
                        return email.to_string();
                    }
                }
            }
        }
    }

    "Unknown".to_string()
}

#[command]
fn get_browser_history() -> Result<Vec<BrowserHistory>, String> {
    let mut all_history = Vec::new();
    let profiles = get_chrome_profiles();
    let profile_names = get_profile_display_names();

    for profile in profiles {
        let profile_name = profile.file_name().unwrap().to_string_lossy().to_string();
        let profile_display_name = profile_names.get(&profile_name).cloned().unwrap_or(profile_name.clone());
        let gmail = get_gmail_for_profile(&profile); // Fetch Gmail ID

        let history_path = profile.join("History");
        let temp_path = env::temp_dir().join(format!("chrome_history_{}.db", profile_name));

        // Copy history file to avoid lock issues
        if let Err(err) = fs::copy(&history_path, &temp_path) {
            eprintln!("Failed to copy history DB for profile {}: {}", profile_display_name, err);
            continue;
        }

        let conn = match Connection::open(&temp_path) {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Failed to open history DB for profile {}: {}", profile_display_name, err);
                continue;
            }
        };

        let query = "
            SELECT title, url, last_visit_time 
            FROM urls 
            ORDER BY last_visit_time DESC 
            LIMIT 50
        ";

        let mut stmt = match conn.prepare(query) {
            Ok(stmt) => stmt,
            Err(err) => {
                eprintln!("Failed to prepare query for profile {}: {}", profile_display_name, err);
                continue;
            }
        };

        let history_iter = match stmt.query_map([], |row| {
            let raw_time: i64 = row.get(2)?; 
            let unix_timestamp = (raw_time / 1_000_000) - 11_644_473_600;
            let datetime_utc = NaiveDateTime::from_timestamp_opt(unix_timestamp, 0)
                .map(|dt| DateTime::<Utc>::from_utc(dt, Utc));

            let local_time = datetime_utc
                .map(|dt| dt.with_timezone(&Local))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown Time".to_string());

            Ok(BrowserHistory {
                profile: profile_name.clone(),
                profile_display_name: profile_display_name.clone(),
                gmail: gmail.clone(),
                title: row.get(0)?,
                url: row.get(1)?,
                visit_time: local_time,
            })
        }) {
            Ok(iter) => iter,
            Err(err) => {
                eprintln!("Failed to execute query for profile {}: {}", profile_display_name, err);
                continue;
            }
        };

        for entry in history_iter {
            match entry {
                Ok(history) => all_history.push(history),
                Err(err) => eprintln!("Error reading history entry for profile {}: {}", profile_display_name, err),
            }
        }
    }

    Ok(all_history)
}

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
        .invoke_handler(tauri::generate_handler![get_ram_usage, get_running_apps, get_installed_apps, get_browser_history])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

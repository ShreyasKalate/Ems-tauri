use rusqlite::{Connection, Result};
use serde::{Serialize, Deserialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct BrowserHistory {
    profile: String,
    profile_display_name: String,
    gmail: String,
    title: String,
    url: String,
    visit_time: String,
}

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

#[tauri::command]
pub fn get_browser_history() -> Result<Vec<BrowserHistory>, String> {
    let mut all_history = Vec::new();
    let profiles = get_chrome_profiles();
    let profile_names = get_profile_display_names();

    for profile in profiles {
        let profile_name = profile.file_name().unwrap().to_string_lossy().to_string();
        let profile_display_name = profile_names.get(&profile_name).cloned().unwrap_or(profile_name.clone());
        let gmail = get_gmail_for_profile(&profile);

        let history_path = profile.join("History");
        let temp_path = env::temp_dir().join(format!("chrome_history_{}.db", profile_name));

        // Copy history file to temporary location to avoid lock issues
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

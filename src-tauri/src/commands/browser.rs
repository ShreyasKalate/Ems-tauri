use rusqlite::{Connection, Result};
use serde::{Serialize, Deserialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use chrono::{DateTime, Local, NaiveDateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct BrowserHistory {
    profile: String,
    browser: String,
    profile_display_name: String,
    gmail: String,
    title: String,
    url: String,
    visit_time: String,
}

fn get_browser_profiles(base_path: &str, browser_name: &str) -> Vec<(PathBuf, String, String)> {
    let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    let browser_base_path = PathBuf::from(format!("{}{}", user_profile, base_path));

    let mut profiles = Vec::new();
    if let Ok(entries) = fs::read_dir(browser_base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let history_db = path.join("History");
                if history_db.exists() {
                    let profile_name = entry.file_name().to_string_lossy().to_string();
                    profiles.push((path, profile_name, browser_name.to_string()));
                }
            }
        }
    }
    profiles
}

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

fn extract_history(profiles: Vec<(PathBuf, String, String)>) -> Vec<BrowserHistory> {
    let mut all_history = Vec::new();

    for (profile, profile_display_name, browser_name) in profiles {
        let gmail = get_gmail_for_profile(&profile);

        let history_path = if browser_name == "Firefox" {
            profile.join("places.sqlite")
        } else {
            profile.join("History") //for chromium based browsers
        };

        let temp_path = env::temp_dir().join(format!("{}_history_{}.db", browser_name.to_lowercase(), profile_display_name));

        if let Err(err) = fs::copy(&history_path, &temp_path) {
            eprintln!("Failed to copy history DB for {} profile {}: {}", browser_name, profile_display_name, err);
            continue;
        }

        let conn = match Connection::open(&temp_path) {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Failed to open history DB for {} profile {}: {}", browser_name, profile_display_name, err);
                continue;
            }
        };

        let query = if browser_name == "Firefox" {
            "SELECT title, url, visit_date / 1000000 AS visit_time FROM moz_places 
            JOIN moz_historyvisits ON moz_places.id = moz_historyvisits.place_id 
            ORDER BY visit_time DESC LIMIT 50"
        } else {
            "SELECT title, url, last_visit_time 
            FROM urls 
            ORDER BY last_visit_time DESC 
            LIMIT 50"
        };

        let mut stmt = match conn.prepare(query) {
            Ok(stmt) => stmt,
            Err(err) => {
                eprintln!("Failed to prepare query for {} profile {}: {}", browser_name, profile_display_name, err);
                continue;
            }
        };

        let history_iter = match stmt.query_map([], |row| {
            let raw_time: i64 = row.get(2)?;
            let unix_timestamp = if browser_name == "Firefox" {
                raw_time
            } else {
                (raw_time / 1_000_000) - 11_644_473_600
            };

            let datetime_utc = NaiveDateTime::from_timestamp_opt(unix_timestamp, 0)
                .map(|dt| DateTime::<Utc>::from_utc(dt, Utc));

            let local_time = datetime_utc
                .map(|dt| dt.with_timezone(&Local))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown Time".to_string());

            Ok(BrowserHistory {
                profile: profile_display_name.clone(),
                browser: browser_name.clone(),
                profile_display_name: profile_display_name.clone(),
                gmail: gmail.clone(),
                title: row.get(0)?,
                url: row.get(1)?,
                visit_time: local_time,
            })
        }) {
            Ok(iter) => iter,
            Err(err) => {
                eprintln!("Failed to execute query for {} profile {}: {}", browser_name, profile_display_name, err);
                continue;
            }
        };

        for entry in history_iter {
            match entry {
                Ok(history) => all_history.push(history),
                Err(err) => eprintln!("Error reading history entry for {} profile {}: {}", browser_name, profile_display_name, err),
            }
        }
    }

    all_history
}

#[tauri::command]
pub fn get_browser_history() -> Result<Vec<BrowserHistory>, String> {
    let mut all_history = Vec::new();

    let chrome_profiles = get_browser_profiles("\\AppData\\Local\\Google\\Chrome\\User Data", "Chrome");
    all_history.extend(extract_history(chrome_profiles));

    let brave_profiles = get_browser_profiles("\\AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data", "Brave");
    all_history.extend(extract_history(brave_profiles));

    let edge_profiles = get_browser_profiles("\\AppData\\Local\\Microsoft\\Edge\\User Data", "Edge");
    all_history.extend(extract_history(edge_profiles));

    let firefox_base_path = PathBuf::from(format!(
        "{}\\AppData\\Roaming\\Mozilla\\Firefox\\Profiles",
        env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    ));

    let firefox_profiles = if let Ok(entries) = fs::read_dir(firefox_base_path) {
        entries.filter_map(|entry| entry.ok().map(|e| (e.path(), e.file_name().to_string_lossy().to_string(), "Firefox".to_string()))).collect()
    } else {
        Vec::new()
    };
    all_history.extend(extract_history(firefox_profiles));

    Ok(all_history)
}

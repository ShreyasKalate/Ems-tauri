use chrono::Utc;
use once_cell::sync::Lazy;
use rusqlite::params;
use std::{collections::HashSet, process::Command, sync::Mutex};
use winreg::{enums::*, RegKey};

use crate::get_conn;

#[derive(Clone, Debug)]
pub struct InstalledApp {
    pub identifying_number: String,
    pub install_date: String,
    pub install_location: String,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub source: String,
    pub timestamp: String,
}

pub static INSTALLED_APPS_CACHE: Lazy<Mutex<Vec<Vec<InstalledApp>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn create_table() {
    let conn = get_conn();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS installed_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            identifying_number TEXT,
            install_date TEXT,
            install_location TEXT,
            vendor TEXT,
            version TEXT,
            source TEXT,
            timestamp TIMESTAMP NOT NULL,
            is_deleted BOOLEAN DEFAULT FALSE,
            UNIQUE(name, version, source)
        )",
        [],
    )
    .expect("❌ Failed to create installed_apps table");
}

fn format_date(date: &str) -> String {
    if date.len() == 8 {
        format!("{}-{}-{}", &date[6..], &date[4..6], &date[0..4])
    } else {
        "N/A".to_string()
    }
}

fn get_msi_apps() -> Vec<(String, String)> {
    let output = Command::new("wmic")
        .args(["product", "get", "IdentifyingNumber,Name"])
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut apps = Vec::new();
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let identifying_number = parts[0].to_string();
                let name = parts[1..].join(" ");
                apps.push((name, identifying_number));
            }
        }
        apps
    } else {
        vec![]
    }
}

fn extract_from_registry(
    key: &RegKey,
    app_list: &mut Vec<InstalledApp>,
    source: &str,
    msi_apps: &[(String, String)],
) {
    for subkey_name in key.enum_keys().filter_map(Result::ok) {
        if let Ok(subkey) = key.open_subkey(&subkey_name) {
            let name = subkey
                .get_value::<String, _>("DisplayName")
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }

            let identifying_number = msi_apps
                .iter()
                .find(|(app_name, _)| app_name == &name)
                .map(|(_, id)| id.clone())
                .unwrap_or_else(|| "N/A".into());

            let install_date = subkey
                .get_value::<String, _>("InstallDate")
                .map(|d| format_date(&d))
                .unwrap_or_else(|_| "N/A".into());

            let install_location = subkey
                .get_value::<String, _>("InstallLocation")
                .unwrap_or_else(|_| "N/A".into());

            let vendor = subkey
                .get_value::<String, _>("Publisher")
                .unwrap_or_else(|_| "Unknown".into());

            let version = subkey
                .get_value::<String, _>("DisplayVersion")
                .unwrap_or_else(|_| "Unknown".into());

            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            app_list.push(InstalledApp {
                identifying_number,
                install_date,
                install_location,
                name: name.trim().into(),
                vendor,
                version: version.trim().into(),
                source: source.into(),
                timestamp,
            });
        }
    }
}

pub fn collect_info() -> Vec<InstalledApp> {
    let msi_apps = get_msi_apps();
    let mut all_apps: Vec<InstalledApp> = Vec::new();

    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    {
        extract_from_registry(&hklm, &mut all_apps, "system", &msi_apps);
    }

    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    {
        extract_from_registry(&hkcu, &mut all_apps, "user", &msi_apps);
    }

    all_apps
}

pub fn push_to_cache(snapshot: Vec<InstalledApp>) {
    let mut cache = INSTALLED_APPS_CACHE.lock().unwrap();
    cache.push(snapshot);
}

pub fn flush_cache() {
    let mut cache = INSTALLED_APPS_CACHE.lock().unwrap();
    if cache.is_empty() {
        return;
    }

    let latest = cache.pop().unwrap();
    cache.clear();

    let mut conn = get_conn();
    let tx = conn.transaction().expect("❌ Failed to begin transaction");

    let mut seen_keys = HashSet::new();

    for app in &latest {
        let key = format!("{}|{}|{}", app.name, app.version, app.source);
        seen_keys.insert(key.clone());

        tx.execute(
            "INSERT INTO installed_apps (
                name, identifying_number, install_date, install_location,
                vendor, version, source, timestamp, is_deleted
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, FALSE)
            ON CONFLICT(name, version, source) DO UPDATE SET 
                identifying_number = excluded.identifying_number,
                install_date = excluded.install_date,
                install_location = excluded.install_location,
                vendor = excluded.vendor,
                timestamp = excluded.timestamp,
                is_deleted = FALSE",
            params![
                app.name,
                app.identifying_number,
                app.install_date,
                app.install_location,
                app.vendor,
                app.version,
                app.source,
                app.timestamp,
            ],
        )
        .unwrap();
    }

    let mut stmt = tx
        .prepare("SELECT name, version, source FROM installed_apps WHERE is_deleted = FALSE")
        .unwrap();

    let db_apps = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .unwrap();

    for entry in db_apps.flatten() {
        let key = format!("{}|{}|{}", entry.0, entry.1, entry.2);
        if !seen_keys.contains(&key) {
            tx.execute(
                "UPDATE installed_apps
                 SET is_deleted = TRUE
                 WHERE name = ?1 AND version = ?2 AND source = ?3",
                params![entry.0, entry.1, entry.2],
            )
            .unwrap();
            println!("❌ App removed: {} v{} ({})", entry.0, entry.1, entry.2);
        }
    }

    drop(stmt);

    tx.commit()
        .expect("❌ Failed to commit installed_apps transaction");
    println!("✅ Installed apps flushed to database");
}

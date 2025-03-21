use rusqlite::{params, Connection};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::process::Command;
use std::time::SystemTime;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize, Deserialize, Clone)]
pub struct InstalledApp {
    identifying_number: String,
    install_date: String,
    install_location: String,
    name: String,
    vendor: String,
    version: String,
    source: String, // "system" or "user-shail"
}

fn format_date(date: &str) -> String {
    if date.len() == 8 {
        format!("{}-{}-{}", &date[6..], &date[4..6], &date[0..4])
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

fn extract_from_registry(key: &RegKey, app_list: &mut Vec<InstalledApp>, source: &str, msi_apps: &[(String, String)]) {
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
                name: name.trim().to_string(),
                vendor,
                version: version.trim().to_string(),
                source: source.trim().to_string(),            
            });
        }
    }
}

pub fn store_installed_apps_to_db() {
    let conn = Connection::open("ems_data.db").expect("Failed to open database");
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
            scanned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            is_deleted BOOLEAN DEFAULT FALSE,
            UNIQUE(name, version, source)
        )",
        [],
    ).unwrap();

    let msi_apps = get_msi_installed_apps();
    let mut all_apps: Vec<InstalledApp> = Vec::new();

    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
        extract_from_registry(&hklm, &mut all_apps, "system", &msi_apps);
    }

    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
        extract_from_registry(&hkcu, &mut all_apps, "user-shail", &msi_apps);
    }

    let now = SystemTime::now();
    let mut seen_keys = HashSet::new();

    for app in &all_apps {
        let key = format!("{}|{}|{}", app.name, app.version, app.source);
        seen_keys.insert(key.clone());

        conn.execute(
            "INSERT INTO installed_apps (name, identifying_number, install_date, install_location, vendor, version, source, scanned_at, is_deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, FALSE)
             ON CONFLICT(name, version, source) DO UPDATE SET 
                 identifying_number=excluded.identifying_number,
                 install_date=excluded.install_date,
                 install_location=excluded.install_location,
                 vendor=excluded.vendor,
                 scanned_at=CURRENT_TIMESTAMP,
                 is_deleted=FALSE",
            params![
                app.name,
                app.identifying_number,
                app.install_date,
                app.install_location,
                app.vendor,
                app.version,
                app.source,
            ],
        ).unwrap();
    }

    // Mark missing apps as deleted
    let mut stmt = conn
        .prepare("SELECT name, version, source FROM installed_apps WHERE is_deleted = FALSE")
        .unwrap();

    let db_apps = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let version: String = row.get(1)?;
            let source: String = row.get(2)?;
            Ok((name, version, source))
        })
        .unwrap();

    for entry in db_apps.flatten() {
        let key = format!("{}|{}|{}", entry.0, entry.1, entry.2);
        if !seen_keys.contains(&key) {
            conn.execute(
                "UPDATE installed_apps SET is_deleted = TRUE WHERE name = ?1 AND version = ?2 AND source = ?3",
                params![entry.0, entry.1, entry.2],
            ).unwrap();
            println!("❌ App removed: {} v{} ({})", entry.0, entry.1, entry.2);
        }
    }

    println!("✅ Installed apps scan complete. {} total apps stored.", all_apps.len());
}
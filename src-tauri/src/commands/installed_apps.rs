use crate::commands::database::{execute_write_query, execute_read_query}; 
use rusqlite::ToSql;
use serde::{Deserialize, Serialize};
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

/// Formats date from `yyyymmdd` to `yyyy-mm-dd`
fn format_date(date: &str) -> String {
    if date.len() == 8 {
        format!("{}-{}-{}", &date[6..], &date[4..6], &date[0..4])
    } else {
        "N/A".to_string()
    }
}

/// Gets MSI-installed apps using `wmic`
fn get_msi_installed_apps() -> Vec<(String, String)> {
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
        return apps;
    }
    vec![]
}

/// Extracts installed applications from registry
fn extract_from_registry(
    key: &RegKey,
    app_list: &mut Vec<InstalledApp>,
    source: &str,
    msi_apps: &[(String, String)],
) {
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

            let install_date = subkey
                .get_value::<String, _>("InstallDate")
                .map(|d| format_date(&d))
                .unwrap_or("N/A".to_string());

            let install_location =
                subkey.get_value::<String, _>("InstallLocation").unwrap_or("N/A".to_string());
            let vendor = subkey.get_value::<String, _>("Publisher").unwrap_or("Unknown".to_string());
            let version = subkey
                .get_value::<String, _>("DisplayVersion")
                .unwrap_or("Unknown".to_string());

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

/// Stores installed applications in SQLite database
pub fn store_installed_apps_to_db() {
    // ✅ Create the table if not exists
    let create_table_query = "
        CREATE TABLE IF NOT EXISTS installed_apps (
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
        )";
    execute_write_query(create_table_query, vec![]).expect("Failed to create installed_apps table");

    let msi_apps = get_msi_installed_apps();
    let mut all_apps: Vec<InstalledApp> = Vec::new();

    // ✅ Read installed apps from SYSTEM registry
    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    {
        extract_from_registry(&hklm, &mut all_apps, "system", &msi_apps);
    }

    // ✅ Read installed apps from USER registry
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    {
        extract_from_registry(&hkcu, &mut all_apps, "user-shail", &msi_apps);
    }

    let mut seen_keys = HashSet::new();

    // ✅ Insert or update installed applications
    for app in &all_apps {
        let key = format!("{}|{}|{}", app.name, app.version, app.source);
        seen_keys.insert(key.clone());

        let insert_query = "
            INSERT INTO installed_apps 
            (name, identifying_number, install_date, install_location, vendor, version, source, scanned_at, is_deleted) 
            VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, FALSE) 
            ON CONFLICT(name, version, source) 
            DO UPDATE SET 
                identifying_number = excluded.identifying_number,
                install_date = excluded.install_date,
                install_location = excluded.install_location,
                vendor = excluded.vendor,
                scanned_at = CURRENT_TIMESTAMP,
                is_deleted = FALSE";

        let params: Vec<Box<dyn ToSql + Send + Sync>> = vec![
            Box::new(app.name.clone()),
            Box::new(app.identifying_number.clone()),
            Box::new(app.install_date.clone()),
            Box::new(app.install_location.clone()),
            Box::new(app.vendor.clone()),
            Box::new(app.version.clone()),
            Box::new(app.source.clone()),
        ];

        if let Err(err) = execute_write_query(insert_query, params) {
            eprintln!("❌ Failed to insert/update installed app: {}", err);
        }
    }

    // ✅ Mark missing apps as deleted
    let select_query = "SELECT name, version, source FROM installed_apps WHERE is_deleted = FALSE";
let db_apps: Vec<(String, String, String)> = match execute_read_query(
    select_query, 
    vec![], 
    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
) {
    Ok(rows) => rows,
    Err(err) => {
        eprintln!("❌ Failed to query installed apps: {}", err);
        Vec::new()
    }
};


    for (name, version, source) in db_apps {
        let key = format!("{}|{}|{}", name, version, source);
        if !seen_keys.contains(&key) {
            let update_query = "UPDATE installed_apps SET is_deleted = TRUE WHERE name = ? AND version = ? AND source = ?";
            let params: Vec<Box<dyn ToSql + Send + Sync>> =
                vec![Box::new(name.clone()), Box::new(version.clone()), Box::new(source.clone())];

            if let Err(err) = execute_write_query(update_query, params) {
                eprintln!("❌ Failed to mark app as deleted: {}", err);
            } else {
                println!("❌ App removed: {} v{} ({})", name, version, source);
            }
        }
    }

    println!("✅ Installed apps scan complete. {} total apps stored.", all_apps.len());
}

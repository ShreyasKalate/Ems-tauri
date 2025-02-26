use sysinfo::System;
use serde::{Serialize, Deserialize};
use rusqlite::{Connection, params};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DB_CONN: Mutex<Connection> = Mutex::new(
        Connection::open("ems_data.db").expect("Failed to open database")
    );
    static ref RAM_USAGE_CACHE: Mutex<Vec<f64>> = Mutex::new(Vec::new());
}

#[derive(Serialize, Deserialize)]
pub struct RamUsage {
    timestamp: String,
    min_ram_gb: f64,
    max_ram_gb: f64,
    avg_ram_gb: f64,
    total_ram_gb: f64,
    ram_usage_percent: f64,
}

#[tauri::command]
pub fn get_ram_usage() -> String {
    let conn = DB_CONN.lock().unwrap();

    // Ensure table exists before querying
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ram_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            min_ram_gb REAL,
            max_ram_gb REAL,
            avg_ram_gb REAL,
            total_ram_gb REAL,
            ram_usage_percent REAL
        )",
        [],
    ).expect("Failed to create table");

    // Fetch latest RAM usage
    let mut stmt = conn.prepare(
        "SELECT timestamp, min_ram_gb, max_ram_gb, avg_ram_gb, total_ram_gb, ram_usage_percent
         FROM ram_usage ORDER BY timestamp DESC LIMIT 1"
    ).expect("Failed to prepare query");

    let latest_data: Option<RamUsage> = stmt.query_row([], |row| {
        Ok(RamUsage {
            timestamp: row.get(0)?,
            min_ram_gb: row.get(1)?,
            max_ram_gb: row.get(2)?,
            avg_ram_gb: row.get(3)?,
            total_ram_gb: row.get(4)?,
            ram_usage_percent: row.get(5)?,
        })
    }).ok();

    serde_json::to_string(&latest_data).unwrap_or_else(|_| "{}".to_string())
}

pub fn track_ram_usage() {
    let conn = DB_CONN.lock().unwrap();
    
    // ✅ Ensure table exists before starting RAM tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ram_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            min_ram_gb REAL,
            max_ram_gb REAL,
            avg_ram_gb REAL,
            total_ram_gb REAL,
            ram_usage_percent REAL
        )",
        [],
    ).expect("Failed to create table at startup");

    std::thread::spawn(|| {
        loop {
            let mut sys = System::new_all();
            sys.refresh_memory();

            let _total_ram = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_ram = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

            {
                let mut cache = RAM_USAGE_CACHE.lock().unwrap();
                if cache.len() >= 60 {
                    cache.remove(0); // Keep last 60 seconds
                }
                cache.push(used_ram);
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60)); // Every 1 minute

            let mut cache = RAM_USAGE_CACHE.lock().unwrap();
            if cache.is_empty() {
                continue;
            }

            let min_ram = *cache.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let max_ram = *cache.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let avg_ram = cache.iter().sum::<f64>() / cache.len() as f64;
            let total_ram = System::new_all().total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let ram_percent = (avg_ram / total_ram) * 100.0;

            cache.clear(); // Reset cache after storing

            store_ram_usage(min_ram, max_ram, avg_ram, total_ram, ram_percent);
        }
    });
}

pub fn store_ram_usage(min_ram: f64, max_ram: f64, avg_ram: f64, total_ram: f64, ram_percent: f64) {
    let conn = DB_CONN.lock().unwrap();
    
    // Ensure table exists before inserting data
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ram_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            min_ram_gb REAL,
            max_ram_gb REAL,
            avg_ram_gb REAL,
            total_ram_gb REAL,
            ram_usage_percent REAL
        )",
        [],
    ).expect("Failed to create table");

    // Insert the new computed RAM usage
    conn.execute(
        "INSERT INTO ram_usage (min_ram_gb, max_ram_gb, avg_ram_gb, total_ram_gb, ram_usage_percent) 
         VALUES (?, ?, ?, ?, ?)",
        params![min_ram, max_ram, avg_ram, total_ram, ram_percent],
    ).expect("Failed to insert RAM usage data");
}

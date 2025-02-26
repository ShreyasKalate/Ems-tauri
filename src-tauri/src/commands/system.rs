use sysinfo::System;
use serde::{Serialize, Deserialize};
use rusqlite::{Connection, params};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DB_CONN: Mutex<Connection> = Mutex::new(
        Connection::open("ems_data.db").expect("Failed to open database")
    );
}

#[derive(Serialize, Deserialize)]
pub struct RamUsage {
    total_ram_gb: f64,
    used_ram_gb: f64,
}

#[tauri::command]
pub fn get_ram_usage() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    let ram_usage = RamUsage {
        total_ram_gb: total_memory,
        used_ram_gb: used_memory,
    };

    // Store in SQLite
    store_ram_usage(total_memory, used_memory);

    serde_json::to_string(&ram_usage).unwrap_or_else(|_| "{}".to_string())
}

fn store_ram_usage(total_ram: f64, used_ram: f64) {
    let conn = DB_CONN.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ram_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            total_ram_gb REAL,
            used_ram_gb REAL
        )",
        [],
    ).expect("Failed to create table");

    conn.execute(
        "INSERT INTO ram_usage (total_ram_gb, used_ram_gb) VALUES (?, ?)",
        params![total_ram, used_ram],
    ).expect("Failed to insert RAM usage data");
}

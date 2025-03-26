use sysinfo::System;
use rusqlite::params;
use std::{sync::Mutex, time::Duration, thread::sleep};

use crate::commands::database::DB_CONN;

lazy_static::lazy_static! {
    static ref RAM_USAGE_CACHE: Mutex<Vec<f64>> = Mutex::new(Vec::new());
}

/// Called from `database.rs`
pub fn create_ram_usage_table() {
    let conn = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
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
    ).expect("Failed to create ram_usage table");
}

/// Called from `main.rs` directly in a thread
pub fn track_ram_usage() {
    loop {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let used_ram = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        let mut cache = RAM_USAGE_CACHE.lock().unwrap();
        if cache.len() >= 10 {
            cache.remove(0);
        }
        cache.push(used_ram);

        sleep(Duration::from_secs(5));
    }
}

/// Called from `database.rs`
pub fn ram_usage_db() {
    let mut cache = RAM_USAGE_CACHE.lock().unwrap();
    if cache.len() < 10 {
        return;
    }

    let min = *cache.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max = *cache.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let avg = cache.iter().sum::<f64>() / cache.len() as f64;
    let total = System::new_all().total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let percent = (avg / total) * 100.0;

    cache.clear();

    let conn = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
    conn.execute(
        "INSERT INTO ram_usage (min_ram_gb, max_ram_gb, avg_ram_gb, total_ram_gb, ram_usage_percent)
         VALUES (?, ?, ?, ?, ?)",
        params![min, max, avg, total, percent],
    ).expect("Failed to insert RAM data");
}

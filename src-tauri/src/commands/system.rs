use sysinfo::System;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use chrono::Utc;
use crate::commands::database::{execute_write_query, execute_read_query};
use rusqlite::types::ToSql;

lazy_static::lazy_static! {
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

// ✅ Fetch latest RAM usage from the database
#[tauri::command]
pub fn get_ram_usage() -> String {
    let query = "
        SELECT timestamp, min_ram_gb, max_ram_gb, avg_ram_gb, total_ram_gb, ram_usage_percent
        FROM ram_usage ORDER BY timestamp DESC LIMIT 1";

    let result: Result<Vec<RamUsage>, String> = execute_read_query(query, vec![], |row| {
        Ok(RamUsage {
            timestamp: row.get(0)?,
            min_ram_gb: row.get(1)?,
            max_ram_gb: row.get(2)?,
            avg_ram_gb: row.get(3)?,
            total_ram_gb: row.get(4)?,
            ram_usage_percent: row.get(5)?,
        })
    });

    let latest_data = result.ok().and_then(|mut rows| rows.pop());

    serde_json::to_string(&latest_data).unwrap_or_else(|_| "{}".to_string())
}

// ✅ Track RAM usage and store it periodically
pub fn track_ram_usage() {
    thread::spawn(|| {
        let mut sys = System::new_all();

        loop {
            sys.refresh_memory();

            let used_ram = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

            {
                let mut cache = RAM_USAGE_CACHE.lock().unwrap();
                if cache.len() >= 60 {
                    cache.remove(0);
                }
                cache.push(used_ram);
            }

            thread::sleep(Duration::from_secs(1));
        }
    });

    thread::spawn(|| {
        let sys = System::new_all();

        loop {
            thread::sleep(Duration::from_secs(60));

            let mut cache = RAM_USAGE_CACHE.lock().unwrap();
            if cache.is_empty() {
                continue;
            }

            let min_ram = *cache.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let max_ram = *cache.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let avg_ram = cache.iter().sum::<f64>() / cache.len() as f64;
            let total_ram = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let ram_percent = (avg_ram / total_ram) * 100.0;
            let timestamp = Utc::now().to_string();

            cache.clear();

            store_ram_usage(timestamp, min_ram, max_ram, avg_ram, total_ram, ram_percent);
        }
    });
}

// ✅ Store RAM usage in the database
fn store_ram_usage(timestamp: String, min_ram: f64, max_ram: f64, avg_ram: f64, total_ram: f64, ram_percent: f64) {
    let create_table_query = "
        CREATE TABLE IF NOT EXISTS ram_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            min_ram_gb REAL,
            max_ram_gb REAL,
            avg_ram_gb REAL,
            total_ram_gb REAL,
            ram_usage_percent REAL
        )";

    if let Err(err) = execute_write_query(create_table_query, vec![]) {
        eprintln!("❌ Failed to create table: {}", err);
        return;
    }

    let insert_query = "
        INSERT INTO ram_usage (timestamp, min_ram_gb, max_ram_gb, avg_ram_gb, total_ram_gb, ram_usage_percent) 
        VALUES (?, ?, ?, ?, ?, ?)";

    let params: Vec<Box<dyn ToSql + Send + Sync>> = vec![
        Box::new(timestamp),
        Box::new(min_ram),
        Box::new(max_ram),
        Box::new(avg_ram),
        Box::new(total_ram),
        Box::new(ram_percent),
    ];

    if let Err(err) = execute_write_query(insert_query, params) {
        eprintln!("❌ Failed to insert RAM usage data: {}", err);
    } else {
        // println!("✅ RAM usage data stored successfully.");
    }
}

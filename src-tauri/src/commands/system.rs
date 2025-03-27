use sysinfo::{System, Disks};
use rusqlite::params;
use std::{sync::Mutex, time::Duration, thread::sleep};

use crate::commands::database::DB_CONN;

lazy_static::lazy_static! {
    static ref SYSTEM_STATS_CACHE: Mutex<Vec<(f64, f64, f64, f32, f64, f64, f64)>> = Mutex::new(Vec::new());
}

/// Creates `system_stats` table, called from `database.rs`
pub fn create_system_table() {
    let conn = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            used_ram_gb REAL,
            total_ram_gb REAL,
            ram_usage_percent REAL,
            cpu_usage_percent REAL,
            used_disk_gb REAL,
            total_disk_gb REAL,
            disk_usage_percent REAL
        )",
        [],
    ).expect("Failed to create system_stats table");
}

/// Called from `main.rs` — collects system-wide metrics every 5s
pub fn track_system_usage() {
    loop {
        let mut sys = System::new_all();
        sys.refresh_all();

        // RAM
        let used_ram = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_ram = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let ram_percent = if total_ram > 0.0 {
            (used_ram / total_ram) * 100.0
        } else {
            0.0
        };

        // CPU
        let cpu_percent = sys.global_cpu_usage();

        // Disk — find primary (e.g. C:\ or /)
        let disks = Disks::new_with_refreshed_list();
        let mut used_disk = 0.0;
        let mut total_disk = 0.0;

        //windows specific
        for disk in &disks {
            let mount = disk.mount_point().to_string_lossy();
            if cfg!(target_os = "windows") && mount.starts_with("C:") ||
               cfg!(not(target_os = "windows")) && mount == "/" {
                total_disk = disk.total_space() as f64 / 1e9;
                let available = disk.available_space() as f64 / 1e9;
                used_disk = total_disk - available;
                break;
            }
        }

        let disk_percent = if total_disk > 0.0 {
            (used_disk / total_disk) * 100.0
        } else {
            0.0
        };

        {
            let mut cache = SYSTEM_STATS_CACHE.lock().unwrap();
            cache.push((
                used_ram,
                total_ram,
                ram_percent,
                cpu_percent,
                used_disk,
                total_disk,
                disk_percent,
            ));
        }

        sleep(Duration::from_secs(5));
    }
}

/// Called from `database.rs` — drains and stores system-wide stats
pub fn system_db() {
    let mut cache = SYSTEM_STATS_CACHE.lock().unwrap();
    if cache.is_empty() {
        return;
    }

    let rows: Vec<_> = cache.drain(..).collect();
    drop(cache);

    let mut conn = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
    let tx = conn.transaction().expect("❌ Failed to begin transaction");

    for (used_ram, total_ram, ram_percent, cpu_percent, used_disk, total_disk, disk_percent) in rows {
        tx.execute(
            "INSERT INTO system_stats (
                used_ram_gb, total_ram_gb, ram_usage_percent,
                cpu_usage_percent,
                used_disk_gb, total_disk_gb, disk_usage_percent
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                used_ram, total_ram, ram_percent,
                cpu_percent,
                used_disk, total_disk, disk_percent
            ],
        ).expect("❌ Failed to insert system stat row");
    }

    tx.commit().expect("❌ Failed to commit transaction");
}

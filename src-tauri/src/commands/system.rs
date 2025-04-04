use chrono::Utc;
use once_cell::sync::Lazy;
use rusqlite::params;
use std::sync::Mutex;
use sysinfo::{Disks, System};

use crate::get_conn;

#[derive(Clone, Debug)]
pub struct SystemSnapshot {
    pub timestamp: String,
    pub used_ram: f64,
    pub total_ram: f64,
    pub ram_percent: f64,
    pub cpu_percent: f32,
    pub used_disk: f64,
    pub total_disk: f64,
    pub disk_percent: f64,
}

pub static SYSTEM_STATS_CACHE: Lazy<Mutex<Vec<SystemSnapshot>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn create_table() {
    let conn = get_conn();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL,
            used_ram_gb REAL,
            total_ram_gb REAL,
            ram_usage_percent REAL,
            cpu_usage_percent REAL,
            used_disk_gb REAL,
            total_disk_gb REAL,
            disk_usage_percent REAL
        )",
        [],
    )
    .expect("❌ Failed to create system_stats table");
}

fn get_primary_disk(disks: &Disks) -> (f64, f64) {  // explore this
    for disk in disks {
        let mount = disk.mount_point().to_string_lossy();
        if cfg!(target_os = "windows") && mount.starts_with("C:")
            || cfg!(not(target_os = "windows")) && mount == "/"
        {
            let total = disk.total_space() as f64 / 1e9;
            let available = disk.available_space() as f64 / 1e9;
            let used = total - available;
            return (used, total);
        }
    }
    (0.0, 0.0)
}

pub fn collect_info() -> SystemSnapshot {
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

    // Disk
    let disks = Disks::new_with_refreshed_list();
    let (used_disk, total_disk) = get_primary_disk(&disks);

    let disk_percent = if total_disk > 0.0 {
        (used_disk / total_disk) * 100.0
    } else {
        0.0
    };
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    SystemSnapshot {
        timestamp,
        used_ram,
        total_ram,
        ram_percent,
        cpu_percent,
        used_disk,
        total_disk,
        disk_percent,
    }
}

pub fn push_to_cache(snapshot: SystemSnapshot) {
    let mut cache = SYSTEM_STATS_CACHE.lock().unwrap();
    cache.push(snapshot);
}

pub fn flush_cache() {
    let mut cache = SYSTEM_STATS_CACHE.lock().unwrap();
    if cache.is_empty() {
        return;
    }

    let rows: Vec<_> = cache.drain(..).collect();
    drop(cache);

    let mut conn = get_conn();
    let tx = conn.transaction().expect("❌ Failed to begin transaction");

    for snap in rows {
        tx.execute(
            "INSERT INTO system_stats (
                timestamp,
                used_ram_gb,
                total_ram_gb,
                ram_usage_percent,
                cpu_usage_percent,
                used_disk_gb,
                total_disk_gb,
                disk_usage_percent
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                snap.timestamp,
                snap.used_ram,
                snap.total_ram,
                snap.ram_percent,
                snap.cpu_percent,
                snap.used_disk,
                snap.total_disk,
                snap.disk_percent
            ],
        )
        .expect("❌ Failed to insert system stat row");
    }

    tx.commit().expect("❌ Failed to commit transaction");
    println!("✅ System stats flushed to database");
}

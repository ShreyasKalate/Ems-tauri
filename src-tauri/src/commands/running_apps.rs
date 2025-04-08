use chrono::{DateTime, Duration, TimeZone, Utc};
use once_cell::sync::Lazy;
use rusqlite::params;
use std::{collections::HashSet, sync::Mutex};
use sysinfo::System;

use crate::get_conn;

#[derive(Clone, Debug)]
pub struct RunningApp {
    pub name: String,
    pub pid: u32,
    pub memory_usage_mb: f64,
    pub start_time: String,
    pub running_time: String,
    pub last_updated: String,
}

pub static EMS_LAUNCH_TIME: Lazy<i64> = Lazy::new(|| Utc::now().timestamp());
pub static RUNNING_APPS_CACHE: Lazy<Mutex<Vec<RunningApp>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn create_table() {
    let conn = get_conn();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS running_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            pid INTEGER,
            memory_usage_mb REAL,
            start_time TEXT,
            running_time TEXT,
            last_updated TEXT,
            is_terminated BOOLEAN DEFAULT FALSE,
            UNIQUE(pid, start_time)
        )",
        [],
    )
    .expect("❌ Failed to create running_apps table");
}

pub fn collect_info() -> Vec<RunningApp> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let now = Utc::now();
    let now_ts = now.timestamp();

    let mut result = Vec::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();
        let process_start_ts = process.start_time() as i64;

        let adjusted_start_ts =
            if process_start_ts == 0 || process_start_ts < (*EMS_LAUNCH_TIME - 60) {
                *EMS_LAUNCH_TIME
            } else {
                process_start_ts
            };

        let dt_start: DateTime<Utc> = Utc.timestamp_opt(adjusted_start_ts, 0).unwrap();
        let start_time = dt_start.format("%Y-%m-%d %H:%M:%S").to_string();

        let duration = Duration::seconds(now_ts - adjusted_start_ts);
        let running_time = format!(
            "{:02}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        );

        let last_updated = now.format("%Y-%m-%d %H:%M:%S").to_string();

        result.push(RunningApp {
            name,
            pid: pid.as_u32(),
            memory_usage_mb: process.memory() as f64 / 1024.0 / 1024.0,
            start_time,
            running_time,
            last_updated,
        });
    }

    result
}

pub fn push_to_cache(new_apps: Vec<RunningApp>) {
    let mut cache = RUNNING_APPS_CACHE.lock().unwrap();
    for app in new_apps {
        match cache
            .iter_mut()
            .find(|a| a.pid == app.pid && a.start_time == app.start_time)
        {
            Some(existing) => {
                existing.memory_usage_mb = app.memory_usage_mb;
                existing.running_time = app.running_time.clone();
                existing.last_updated = app.last_updated.clone();
            }
            None => cache.push(app),
        }
    }
}

pub fn flush_cache() {
    let cache = RUNNING_APPS_CACHE.lock().unwrap();
    if cache.is_empty() {
        return;
    }

    let snapshot = cache.clone();
    drop(cache);

    let mut conn = get_conn();
    let tx = conn.transaction().expect("❌ Failed to begin transaction");

    let mut seen_keys = HashSet::new();

    for app in &snapshot {
        seen_keys.insert((app.pid, app.start_time.clone()));
        tx.execute(
            "INSERT INTO running_apps (
                name, pid, memory_usage_mb,
                start_time, running_time, last_updated, is_terminated
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, FALSE)
            ON CONFLICT(pid, start_time) DO UPDATE SET
                memory_usage_mb = excluded.memory_usage_mb,
                running_time = excluded.running_time,
                last_updated = excluded.last_updated,
                is_terminated = FALSE",
            params![
                app.name,
                app.pid,
                app.memory_usage_mb,
                app.start_time,
                app.running_time,
                app.last_updated,
            ],
        )
        .unwrap();
    }

    // Fallback check for stale processes (not updated recently)
    tx.execute(
        "UPDATE running_apps
         SET is_terminated = TRUE
         WHERE is_terminated = FALSE
         AND strftime('%s', 'now') - strftime('%s', last_updated) > 30",
        [],
    )
    .unwrap();

    tx.commit().
        expect("❌ Failed to commit running_apps");
    println!("✅ Flushed running apps to database");
}

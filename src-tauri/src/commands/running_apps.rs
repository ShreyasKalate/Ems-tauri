use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Asia::Kolkata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use sysinfo::System;

lazy_static::lazy_static! {
    static ref PROCESS_TIMES: Mutex<HashMap<String, (i64, i64)>> = Mutex::new(HashMap::new());
    static ref EMS_LAUNCH_TIME: i64 = Utc::now().timestamp();
}

#[derive(Serialize, Deserialize)]
pub struct RunningApp {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory_usage: u64,
    start_time: String,
    running_time: String,
}

#[tauri::command]
pub fn get_running_apps() -> Vec<RunningApp> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let now_ist = Kolkata
        .from_utc_datetime(&Utc::now().naive_utc())
        .timestamp();

    let mut process_times = PROCESS_TIMES.lock().unwrap();

    sys.processes()
        .iter()
        .map(|(pid, process)| {
            let process_name = process.name().to_string_lossy().to_string();
            let process_start_time = process.start_time() as i64;

            let adjusted_start_time =
                if process_start_time == 0 || process_start_time < (*EMS_LAUNCH_TIME - 60) {
                    *EMS_LAUNCH_TIME
                } else {
                    process_start_time
                };

            let ist_start_time = Kolkata
                .timestamp(adjusted_start_time, 0)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            // Get previous total time and last recorded time
            let (previous_total_time, last_update) = process_times
                .entry(process_name.clone())
                .or_insert((0, now_ist));

            // Correctly track running time **even if the system sleeps**
            let elapsed_time = now_ist - *last_update;

            // Update running time
            *previous_total_time += elapsed_time;

            // Store updated values
            *last_update = now_ist;

            let running_duration = Duration::seconds(*previous_total_time);
            let hours = running_duration.num_hours();
            let minutes = running_duration.num_minutes() % 60;
            let seconds = running_duration.num_seconds() % 60;
            let running_time = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

            RunningApp {
                name: process_name,
                pid: pid.as_u32(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                start_time: ist_start_time,
                running_time,
            }
        })
        .collect()
}

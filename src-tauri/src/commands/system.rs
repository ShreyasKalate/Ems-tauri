use sysinfo::System;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RunningApp {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory_usage: u64,
    start_time: String,
}

#[tauri::command]
pub fn get_running_apps() -> Vec<RunningApp> {
    let mut sys = System::new_all();
    sys.refresh_all();

    sys.processes()
        .iter()
        .map(|(pid, process)| RunningApp {
            name: process.name().to_string_lossy().to_string(),
            pid: pid.as_u32(),
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            start_time: chrono::DateTime::from_timestamp(process.start_time() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or("Unknown".to_string()),
        })
        .collect()
}

#[tauri::command]
pub fn get_ram_usage() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    format!("Total RAM: {:.2} GB\nUsed RAM: {:.2} GB", total_memory, used_memory)
}

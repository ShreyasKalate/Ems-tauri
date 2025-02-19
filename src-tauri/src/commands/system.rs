use sysinfo::System;
use serde::{Serialize, Deserialize};

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

    serde_json::to_string(&ram_usage).unwrap_or_else(|_| "{}".to_string())
}

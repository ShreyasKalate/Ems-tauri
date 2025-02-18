use sysinfo::System;

#[tauri::command]
pub fn get_ram_usage() -> String {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    format!("Total RAM: {:.2} GB\nUsed RAM: {:.2} GB", total_memory, used_memory)
}

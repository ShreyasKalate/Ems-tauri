use sysinfo::System;
use serde::{Serialize, Deserialize};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct VisibleApp {
    name: String,
    pid: u32,
}

#[tauri::command]
pub fn get_visible_apps() -> Vec<VisibleApp> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let mut interactive_pids:HashSet<u32> = HashSet::new();

    unsafe {
        if let Err(e) = EnumWindows(Some(enum_window_proc), LPARAM(&mut interactive_pids as *mut _ as isize)) {
            eprintln!("Failed to enumerate windows: {:?}", e);
        }
    }

    let visible_apps: Vec<VisibleApp> = sys.processes()
        .iter()
        .filter_map(|(&pid, process)| {
            if interactive_pids.contains(&pid.as_u32()) {
                Some(VisibleApp {
                    name: process.name().to_string_lossy().to_string(),
                    pid: pid.as_u32(),
                })
            } else {
                None
            }
        })
        .collect();

    visible_apps
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let interactive_pids = &mut *(lparam.0 as *mut HashSet<u32>);

    if IsWindowVisible(hwnd).as_bool() {
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id != 0 {
            interactive_pids.insert(process_id);
        }
    }
    true.into()
}

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

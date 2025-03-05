use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, GetForegroundWindow
};
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};

lazy_static::lazy_static! {
    static ref PROCESS_TIMES: Mutex<HashMap<String, (i64, i64, bool)>> = Mutex::new(HashMap::new());
    static ref TOP_PROCESS_TIMES: Mutex<HashMap<String, (i64, i64, bool)>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
pub struct VisibleApp {
    name: String,
    pid: u32,
    window_title: String,
    curr_session: String,
    total_usage: String,
    top_usage: String,
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let visible_apps = &mut *(lparam.0 as *mut Vec<VisibleApp>);
    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title);

    if IsWindowVisible(hwnd).as_bool() && len > 0 {
        let window_title = String::from_utf16_lossy(&title[..len as usize]);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let now = Utc::now().timestamp();
        let mut process_times = PROCESS_TIMES.lock().unwrap();
        let mut top_process_times = TOP_PROCESS_TIMES.lock().unwrap();
        
        let (total_time, last_update, is_running) = process_times.entry(window_title.clone()).or_insert((0, now, false));
        if !*is_running {
            *last_update = now;
            *is_running = true;
        }
        let elapsed = now - *last_update;
        *total_time += elapsed;
        *last_update = now;

        // Check if this is the topmost window
        let foreground_hwnd = GetForegroundWindow();
        let is_topmost = hwnd == foreground_hwnd;

        // Track "top usage" time separately
        let (top_time, top_last_update, is_top_running) = top_process_times.entry(window_title.clone()).or_insert((0, now, false));
        if is_topmost {
            if !*is_top_running {
                *top_last_update = now;
                *is_top_running = true;
            }
            let top_elapsed = now - *top_last_update;
            *top_time += top_elapsed;
            *top_last_update = now;
        } else {
            *is_top_running = false;
        }

        visible_apps.push(VisibleApp {
            name: window_title.clone(),
            pid,
            window_title,
            curr_session: format_duration(elapsed),
            total_usage: format_duration(*total_time),
            top_usage: format_duration(*top_time),
        });
    }

    true.into()
}

#[tauri::command]
pub fn get_visible_apps() -> String {
    let mut visible_apps: Vec<VisibleApp> = Vec::new();
    unsafe { EnumWindows(Some(enum_window_proc), LPARAM(&mut visible_apps as *mut _ as isize)); }

    let mut process_times = PROCESS_TIMES.lock().unwrap();
    let mut top_process_times = TOP_PROCESS_TIMES.lock().unwrap();
    let now = Utc::now().timestamp();

    let current_names: Vec<String> = visible_apps.iter().map(|app| app.name.clone()).collect();

    for (name, (total_time, last_update, is_running)) in process_times.iter_mut() {
        if !current_names.contains(name) && *is_running {
            let elapsed = now - *last_update;
            *total_time += elapsed;
            *is_running = false;
        }
    }

    for (name, (top_time, top_last_update, is_top_running)) in top_process_times.iter_mut() {
        if !current_names.contains(name) && *is_top_running {
            let elapsed = now - *top_last_update;
            *top_time += elapsed;
            *is_top_running = false;
        }
    }

    // Add inactive apps to the list
    for (name, (total_time, _, _)) in process_times.iter() {
        let top_time = top_process_times.get(name).map(|(t, _, _)| *t).unwrap_or(0);
        if !current_names.contains(name) {
            visible_apps.push(VisibleApp {
                name: name.clone(),
                pid: 0,
                window_title: String::from("N/A"),
                curr_session: String::from("00:00:00"),
                total_usage: format_duration(*total_time),
                top_usage: format_duration(top_time),
            });
        }
    }

    serde_json::to_string(&visible_apps).unwrap_or_else(|_| "[]".to_string()) // Convert data to JSON format
}

fn format_duration(seconds: i64) -> String {
    let duration = Duration::seconds(seconds);
    format!("{:02}:{:02}:{:02}", duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60)
}

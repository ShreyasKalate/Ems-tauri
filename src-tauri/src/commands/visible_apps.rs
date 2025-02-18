use sysinfo::System;
use serde::{Serialize, Deserialize};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use std::collections::HashSet;
use std::sync::Mutex;
use chrono::{Utc, TimeZone, Duration};
use chrono_tz::Asia::Kolkata;

lazy_static::lazy_static! {
    static ref PROCESS_TIMES: Mutex<std::collections::HashMap<String, (i64, i64)>> = Mutex::new(std::collections::HashMap::new());
    static ref EMS_LAUNCH_TIME: i64 = Utc::now().timestamp();
}

#[derive(Serialize, Deserialize)]
pub struct VisibleApp {
    name: String,
    pid: u32,
    running_time: String,  // ✅ Added running time
}

#[tauri::command]
pub fn get_visible_apps() -> Vec<VisibleApp> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let now_ist = Kolkata.from_utc_datetime(&Utc::now().naive_utc()).timestamp();
    let mut interactive_pids: HashSet<u32> = HashSet::new();

    unsafe {
        if let Err(e) = EnumWindows(Some(enum_window_proc), LPARAM(&mut interactive_pids as *mut _ as isize)) {
            eprintln!("Failed to enumerate windows: {:?}", e);
        }
    }

    let mut process_times = PROCESS_TIMES.lock().unwrap();

    let visible_apps: Vec<VisibleApp> = sys.processes()
        .iter()
        .filter_map(|(&pid, process)| {
            if interactive_pids.contains(&pid.as_u32()) {
                let process_name = process.name().to_string_lossy().to_string();
                let process_start_time = process.start_time() as i64;

                let adjusted_start_time = if process_start_time == 0 || process_start_time < (*EMS_LAUNCH_TIME - 60) {
                    *EMS_LAUNCH_TIME
                } else {
                    process_start_time
                };

                let (previous_total_time, last_update) = process_times.entry(process_name.clone()).or_insert((0, now_ist));

                let elapsed_time = now_ist - *last_update;
                *previous_total_time += elapsed_time;
                *last_update = now_ist;

                let running_duration = Duration::seconds(*previous_total_time);
                let hours = running_duration.num_hours();
                let minutes = running_duration.num_minutes() % 60;
                let seconds = running_duration.num_seconds() % 60;
                let running_time = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

                Some(VisibleApp {
                    name: process_name,
                    pid: pid.as_u32(),
                    running_time,  // ✅ Now includes running time
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

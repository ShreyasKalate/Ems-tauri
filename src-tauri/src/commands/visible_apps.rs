use std::process::Command;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref PROCESS_TIMES: Mutex<HashMap<u32, (i64, i64)>> = Mutex::new(HashMap::new());
    static ref EMS_LAUNCH_TIME: i64 = Utc::now().timestamp();
}

#[derive(Serialize, Deserialize)]
pub struct VisibleAppRaw {
    #[serde(alias = "ProcessName")]
    name: String,
    #[serde(alias = "Id")]
    pid: u32,
    #[serde(alias = "MainWindowTitle")]
    window_title: String,
}

#[derive(Serialize, Deserialize)]
pub struct VisibleApp {
    name: String,
    pid: u32,
    window_title: String,
    running_time: String,
}

#[tauri::command]
pub fn get_visible_apps() -> Vec<VisibleApp> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Get-Process | Where-Object { $_.MainWindowTitle -ne \"\" } | Select-Object ProcessName, Id, MainWindowTitle | ConvertTo-Json -Compress"])
        .output()
        .expect("Failed to execute PowerShell command");

    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        return vec![];
    }

    let json_output = String::from_utf8_lossy(&output.stdout);
    let raw_apps: Vec<VisibleAppRaw> = match serde_json::from_str(&json_output) {
        Ok(apps) => apps,
        Err(err) => {
            eprintln!("Failed to parse JSON: {}", err);
            return vec![];
        }
    };

    let now_ist = Utc::now().timestamp();
    let mut process_times = PROCESS_TIMES.lock().unwrap();

    raw_apps.into_iter().map(|app| {
        let (total_time, last_update) = process_times.entry(app.pid).or_insert((0, now_ist));
        let elapsed_time = now_ist - *last_update;
        *total_time += elapsed_time;
        *last_update = now_ist;

        let running_duration = Duration::seconds(*total_time);
        let hours = running_duration.num_hours();
        let minutes = running_duration.num_minutes() % 60;
        let seconds = running_duration.num_seconds() % 60;

        VisibleApp {
            name: app.name,
            pid: app.pid,
            window_title: app.window_title,
            running_time: format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        }
    }).collect()
}

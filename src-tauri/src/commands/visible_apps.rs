use rusqlite::{params, Connection};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use std::sync::Mutex;
use chrono::Utc;
use serde::{Serialize, Deserialize};

lazy_static::lazy_static! {
    static ref DB_CONN: Mutex<Connection> = Mutex::new(
        Connection::open("ems_data.db").expect("Failed to open database")
    );
}

#[derive(Serialize, Deserialize)]
struct VisibleApp {
    pid: u32,
    name: String,
    window_title: String,
    curr_session: i64,
    total_usage: i64,
    top_usage: i64,
}

pub fn init_db() {
    let conn = DB_CONN.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS visible_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pid INTEGER,
            name TEXT,
            window_title TEXT,
            curr_session INTEGER, 
            total_usage INTEGER, 
            top_usage INTEGER, 
            last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(pid, name, window_title)
        )",
        [],
    ).expect("Failed to create visible_apps table");
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let visible_apps = &mut *(lparam.0 as *mut Vec<VisibleApp>);
    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title);

    if IsWindowVisible(hwnd).as_bool() && len > 0 {
        let window_title = String::from_utf16_lossy(&title[..len as usize]);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        visible_apps.push(VisibleApp {
            pid,
            name: window_title.clone(),
            window_title,
            curr_session: 0,
            total_usage: 0,
            top_usage: 0,
        });
    }
    true.into()
}

pub fn update_visible_apps_db() {
    let conn = DB_CONN.lock().unwrap();
    let mut visible_apps: Vec<VisibleApp> = Vec::new();
    unsafe { EnumWindows(Some(enum_window_proc), LPARAM(&mut visible_apps as *mut _ as isize)); }

    let now = Utc::now().timestamp();
    for app in visible_apps {
        conn.execute(
            "INSERT INTO visible_apps (pid, name, window_title, curr_session, total_usage, top_usage, last_seen) 
             VALUES (?1, ?2, ?3, 0, 0, 0, CURRENT_TIMESTAMP) 
             ON CONFLICT(pid, name, window_title) DO UPDATE 
             SET curr_session = curr_session + 1, 
                 total_usage = total_usage + 1, 
                 top_usage = CASE WHEN ?4 THEN top_usage + 1 ELSE top_usage END, 
                 last_seen = CURRENT_TIMESTAMP",
            params![app.pid, app.name, app.window_title, is_topmost_window(app.pid)],
        ).expect("Failed to insert/update visible app");

        println!(
            "Tracking: PID={} Name={} Title={} Session={} Total={} Top={} Time={}",
            app.pid, app.name, app.window_title, 1, 1, is_topmost_window(app.pid) as i64, now
        );
    }
}

/// **Checks if the window is the topmost active window**
fn is_topmost_window(pid: u32) -> bool {
    unsafe {
        let foreground_hwnd = GetForegroundWindow();
        let mut foreground_pid = 0;
        GetWindowThreadProcessId(foreground_hwnd, Some(&mut foreground_pid));
        pid == foreground_pid
    }
}

pub fn track_visible_apps() {
    init_db();
    std::thread::spawn(|| loop {
        update_visible_apps_db();
        std::thread::sleep(std::time::Duration::from_secs(1)); // Auto-update every second
    });
}

use once_cell::sync::Lazy;
use rusqlite::params;
use std::{collections::HashMap, sync::Mutex};
use sysinfo::{Pid, System};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::WindowsAndMessaging::*,
};

use crate::get_conn;

#[derive(Clone, Debug)]
pub struct VisibleWindow {
    pub app: String,
    pub window_title: String,
    pub pid: u32,
}

#[derive(Default, Clone, Debug)]
pub struct WindowStat {
    pub pid: u32, // Used for visible_windows
    pub total_usage: u64,
    pub top_usage: u64,
}

pub static APP_USAGE_CACHE: Lazy<Mutex<HashMap<String, WindowStat>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static WINDOW_USAGE_CACHE: Lazy<Mutex<HashMap<(String, String), WindowStat>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn create_table() {
    let conn = get_conn();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS visible_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app TEXT UNIQUE,
            total_usage INTEGER DEFAULT 0,
            top_usage INTEGER DEFAULT 0
        )",
        [],
    )
    .expect("❌ Failed to create visible_apps table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS visible_windows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app TEXT,
            window_title TEXT,
            pid INTEGER,
            total_usage INTEGER DEFAULT 0,
            top_usage INTEGER DEFAULT 0,
            UNIQUE(app, window_title)
        )",
        [],
    )
    .expect("❌ Failed to create visible_windows table");
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<VisibleWindow>);
    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }

    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title);
    if len == 0 {
        return true.into();
    }

    let window_title = String::from_utf16_lossy(&title[..len as usize]);
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    let app = get_process_name(pid);
    if !app.is_empty() {
        windows.push(VisibleWindow {
            app,
            window_title,
            pid,
        });
    }

    true.into()
}

fn get_process_name(pid: u32) -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.process(Pid::from(pid as usize))
        .map(|p| {
            let full = p.name().to_string_lossy().to_string();
            full.split('.').next().unwrap_or("").to_lowercase()
        })
        .unwrap_or_default()
}

pub fn collect_info() -> Vec<VisibleWindow> {
    let mut windows = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut windows as *mut _ as isize),
        );
    }
    windows
}

fn is_topmost_window(pid: u32) -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut fg_pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut fg_pid));
        pid == fg_pid
    }
}

pub fn push_to_cache(windows: Vec<VisibleWindow>) {
    let mut app_cache = APP_USAGE_CACHE.lock().unwrap();
    let mut window_cache = WINDOW_USAGE_CACHE.lock().unwrap();

    let mut seen_apps: HashMap<String, bool> = HashMap::new();
    let mut app_top: HashMap<String, bool> = HashMap::new();

    for win in &windows {
        let is_top = is_topmost_window(win.pid);
        let top = if is_top { 1 } else { 0 };

        // Per-window
        window_cache
            .entry((win.app.clone(), win.window_title.clone()))
            .and_modify(|entry| {
                entry.total_usage += 1;
                entry.top_usage += top;
                entry.pid = win.pid;
            })
            .or_insert(WindowStat {
                pid: win.pid,
                total_usage: 1,
                top_usage: top,
            });

        // Per-app
        seen_apps.insert(win.app.clone(), true);
        if is_top {
            app_top.insert(win.app.clone(), true);
        }
    }

    for app in seen_apps.keys() {
        let top = if app_top.get(app).copied().unwrap_or(false) {
            1
        } else {
            0
        };
        app_cache
            .entry(app.clone())
            .and_modify(|entry| {
                entry.total_usage += 1;
                entry.top_usage += top;
            })
            .or_insert(WindowStat {
                pid: 0,
                total_usage: 1,
                top_usage: top,
            });
    }
}

pub fn flush_cache() {
    let mut app_cache = APP_USAGE_CACHE.lock().unwrap();
    let mut window_cache = WINDOW_USAGE_CACHE.lock().unwrap();

    let mut conn = get_conn();
    let tx = conn.transaction().expect("❌ Failed to start transaction");

    for ((app, title), stat) in window_cache.drain() {
        tx.execute(
            "INSERT INTO visible_windows (app, window_title, pid, total_usage, top_usage)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(app, window_title) DO UPDATE SET
               total_usage = total_usage + ?4,
               top_usage = top_usage + ?5,
               pid = excluded.pid",
            params![app, title, stat.pid, stat.total_usage, stat.top_usage],
        )
        .unwrap();
    }

    for (app, stat) in app_cache.drain() {
        tx.execute(
            "INSERT INTO visible_apps (app, total_usage, top_usage)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(app) DO UPDATE SET
               total_usage = total_usage + ?2,
               top_usage = top_usage + ?3",
            params![app, stat.total_usage, stat.top_usage],
        )
        .unwrap();
    }

    tx.commit().expect("❌ Failed to commit aggregated usage");
}

use crate::commands::database::execute_write_query;
use chrono::{DateTime, Duration as ChronoDuration, Local};
use device_query::{DeviceQuery, DeviceState};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::command;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

#[derive(Debug)]
struct AfkState {
    last_activity: Instant,
    afk_start: Option<DateTime<Local>>,
    total_afk_duration: ChronoDuration,
    curr_afk_session: ChronoDuration,
    last_mouse_pos: (i32, i32),
    is_afk: bool,
    current_afk_id: Option<i64>,
}

impl AfkState {
    fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            afk_start: None,
            total_afk_duration: ChronoDuration::zero(),
            curr_afk_session: ChronoDuration::zero(),
            last_mouse_pos: (0, 0),
            is_afk: false,
            current_afk_id: None,
        }
    }
}

static AFK_STATE: Lazy<Arc<Mutex<AfkState>>> = Lazy::new(|| Arc::new(Mutex::new(AfkState::new())));

/// **Initializes the AFK tracking table**
pub fn init_afk_db() {
    let create_table_query = "
        CREATE TABLE IF NOT EXISTS afk_tracking (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            curr_afk_session INTEGER,  
            last_active TIMESTAMP,     
            total_afk_duration INTEGER, 
            afk_start TIMESTAMP
        );
    ";
    

}

/// **Starts the AFK tracker**
pub fn start_afk_tracker() {
    println!("🔥 Starting AFK Tracker...");

    init_afk_db();
    let afk_state = Arc::clone(&AFK_STATE);
    let idle_threshold = Duration::from_secs(10); // 10 seconds
    let device_state = DeviceState::new();

    thread::spawn(move || {
        println!("🟢 AFK Tracker Thread Running...");
        loop {
            let keys = device_state.get_keys();
            let mouse = device_state.get_mouse();
            let idle_time = get_idle_time();
            let now = Local::now();

            let mut state = afk_state.lock().unwrap();
            let new_mouse_pos = (mouse.coords.0, mouse.coords.1);

            let has_user_activity = !keys.is_empty()
                || new_mouse_pos != state.last_mouse_pos
                || mouse.button_pressed.iter().any(|&b| b);

            if has_user_activity {
                if state.is_afk {
                    let session_duration = now.signed_duration_since(state.afk_start.unwrap());
                    state.total_afk_duration = state.total_afk_duration + session_duration;

                    println!(
                        "✅ User active again! Session: {:?} | Total AFK: {:?}",
                        session_duration, state.total_afk_duration
                    );

                    if let Some(afk_id) = state.current_afk_id {
                        finalize_afk_session(afk_id, state.total_afk_duration.num_seconds());
                    }

                    state.is_afk = false;
                    state.afk_start = None;
                    state.curr_afk_session = ChronoDuration::zero();
                    state.current_afk_id = None;
                }
                state.last_activity = Instant::now();
            } else if idle_time >= idle_threshold {
                if !state.is_afk {
                    state.afk_start = Some(now);
                    state.is_afk = true;
                    println!("🚨 AFK Started at {}", now);

                    state.current_afk_id = insert_afk_session(
                        state.curr_afk_session.num_seconds(),  // Store correct AFK session duration
                        state.total_afk_duration.num_seconds(),  // Store correct total AFK duration
                        now
                    );
                    
                } else if let Some(afk_id) = state.current_afk_id {
                    state.curr_afk_session = now.signed_duration_since(state.afk_start.unwrap());

                    println!(
                        "⏳ Still AFK | Session: {}s | Total AFK: {}s",
                        state.curr_afk_session.num_seconds(),
                        state.total_afk_duration.num_seconds()
                    );

                    update_afk_session(afk_id, state.curr_afk_session.num_seconds());
                }
            }

            state.last_mouse_pos = new_mouse_pos;

            thread::sleep(Duration::from_secs(1));
        }
    });

    println!("✅ AFK Tracker Initialized!");
}

/// **Inserts a new AFK session when AFK starts**
fn insert_afk_session(curr_afk: i64, total_afk: i64, afk_start: DateTime<Local>) -> Option<i64> {
    let query = "
        INSERT INTO afk_tracking (curr_afk_session, last_active, total_afk_duration, afk_start) 
        VALUES (?, ?, ?, ?);
    ";

    let afk_start_str = afk_start.format("%Y-%m-%d %H:%M:%S").to_string();
    let last_active_str = afk_start_str.clone(); // Initially same as afk_start

    match execute_write_query(
        query,
        vec![
            Box::new(curr_afk),        // Correct AFK session time
            Box::new(last_active_str), // Last active timestamp
            Box::new(total_afk),       // Correct total AFK duration
            Box::new(afk_start_str),   // AFK start timestamp
        ],
    ) {
        Ok(id) => Some(id),
        Err(err) => {
            eprintln!("❌ Failed to insert AFK session: {:?}", err);
            None
        }
    }
}


fn update_afk_session(afk_id: i64, curr_afk: i64) {
    let query = "
        UPDATE afk_tracking 
        SET curr_afk_session = ? 
        WHERE id = ?;
    ";

    if let Err(e) = execute_write_query(query, vec![Box::new(curr_afk), Box::new(afk_id)]) {
        eprintln!("Error updating AFK session: {:?}", e);
    }
}

fn finalize_afk_session(afk_id: i64, total_afk: i64) {
    let query = "
        UPDATE afk_tracking 
        SET total_afk_duration = ? 
        WHERE id = ?;
    ";

    if let Err(e) = execute_write_query(query, vec![Box::new(total_afk), Box::new(afk_id)]) {
        eprintln!("Error finalizing AFK session: {:?}", e);
    }
}


/// **Gets AFK status**
#[command]
pub fn get_afk_status() -> String {
    let state = AFK_STATE.lock().unwrap();
    let afk_data = format!(
        "📡 AFK Status: is_afk={} | last_active={}s | afk_start={:?} | afk_session={}s | total_afk_time={}s",
        state.is_afk,
        state.last_activity.elapsed().as_secs(),
        state.afk_start,
        state.curr_afk_session.num_seconds(),
        state.total_afk_duration.num_seconds()
    );
    println!("{}", afk_data);
    afk_data
}

/// **Gets system idle time**
fn get_idle_time() -> Duration {
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        if GetLastInputInfo(&mut lii).as_bool() {
            let uptime = GetTickCount64();
            let idle_time = uptime - lii.dwTime as u64;
            Duration::from_millis(idle_time)
        } else {
            Duration::from_secs(0)
        }
    }
}

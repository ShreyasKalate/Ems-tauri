use chrono::{DateTime, Duration as ChronoDuration, Local};
use device_query::{DeviceQuery, DeviceState};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::command;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AfkData {
    last_active: String,
    afk_start: Option<String>,
    afk_session: String,  
    total_afk_duration: String,  
    is_afk: bool,
}

#[derive(Debug)]
struct AfkState {
    last_activity: Instant,
    afk_start: Option<DateTime<Local>>,
    total_afk_duration: ChronoDuration,  
    afk_session: ChronoDuration,  
    last_mouse_pos: (i32, i32),  
    is_afk: bool,
}

impl AfkState {
    fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            afk_start: None,
            total_afk_duration: ChronoDuration::zero(),
            afk_session: ChronoDuration::zero(),
            last_mouse_pos: (0, 0),
            is_afk: false,
        }
    }
}

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

static AFK_STATE: once_cell::sync::Lazy<Arc<Mutex<AfkState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(AfkState::new())));

pub fn start_afk_tracker() {
    let afk_state = Arc::clone(&AFK_STATE);
    let idle_threshold = Duration::from_secs(10); // 10 seconds
    let device_state = DeviceState::new();

    thread::spawn(move || loop {
        let keys = device_state.get_keys();
        let mouse = device_state.get_mouse();
        let idle_time = get_idle_time();
        let now = Local::now();

        let mut state = afk_state.lock().unwrap();
        let new_mouse_pos = (mouse.coords.0, mouse.coords.1);  

        let has_user_activity = 
            !keys.is_empty() || 
            new_mouse_pos != state.last_mouse_pos ||  
            mouse.button_pressed.iter().any(|&b| b); 

        if has_user_activity {
            if state.is_afk {
                let session_duration = now.signed_duration_since(state.afk_start.unwrap());
                state.total_afk_duration = state.total_afk_duration + session_duration;  // ✅ Add only once per session

                println!(
                    "✅ User returned! AFK session: {:?}, Total AFK Time: {:?}",
                    session_duration, state.total_afk_duration
                );

                // 🛠 Reset only `afk_session`, keep `total_afk_duration`
                state.is_afk = false;
                state.afk_start = None;
                state.afk_session = ChronoDuration::zero();
            }
            state.last_activity = Instant::now();
        } 
        // 🚨 Detect AFK when idle time exceeds threshold
        else if idle_time >= idle_threshold {
            if !state.is_afk {
                state.afk_start = Some(now);
                state.is_afk = true;
                println!("🚨 AFK Triggered! Threshold reached.");
            } else {
                // ✅ Keep increasing session time but **do not reset total time**
                let session_duration = now.signed_duration_since(state.afk_start.unwrap());
                state.afk_session = session_duration;

                println!(
                    "⏳ Still AFK | AFK Session: {:?} | Total AFK Time: {:?}",
                    state.afk_session, state.total_afk_duration
                );
            }
        }

        // ✅ Update last known mouse position after processing
        state.last_mouse_pos = new_mouse_pos;  

        println!(
            "🕒 Idle time: {:?}, is_afk: {}, AFK Session: {}s, Total AFK Time: {}s",
            idle_time,
            state.is_afk,
            state.afk_session.num_seconds(),
            state.total_afk_duration.num_seconds()
        );

        thread::sleep(Duration::from_secs(1));
    });
}

#[command]
pub fn get_afk_status() -> AfkData {
    let state = AFK_STATE.lock().unwrap();

    println!(
        "📡 Fetching AFK Status: is_afk={} last_active={}s afk_start={:?} afk_session={}s total_afk_time={}s",
        state.is_afk,
        state.last_activity.elapsed().as_secs(),
        state.afk_start,
        state.afk_session.num_seconds(),
        state.total_afk_duration.num_seconds()
    );

    AfkData {
        last_active: state.last_activity.elapsed().as_secs().to_string(),
        afk_start: state.afk_start.map(|t| t.to_string()),
        afk_session: format!("{}s", state.afk_session.num_seconds()),  
        total_afk_duration: format!("{}s", state.total_afk_duration.num_seconds()),  
        is_afk: state.is_afk,
    }
}

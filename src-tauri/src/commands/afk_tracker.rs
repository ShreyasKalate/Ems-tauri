use chrono::{DateTime, Local};
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
    afk_end: Option<String>,
    afk_duration: Option<String>,
    is_afk: bool,
}

#[derive(Debug)]
struct AfkState {
    last_activity: Instant,
    afk_start: Option<DateTime<Local>>,
    afk_end: Option<DateTime<Local>>,
    is_afk: bool,
}

impl AfkState {
    fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            afk_start: None,
            afk_end: None,
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
    
        let mut state = afk_state.lock().unwrap();
    
        // ✅ If user is active, reset AFK state
        if !keys.is_empty() || mouse.button_pressed.iter().any(|&b| b) {
            if state.is_afk {
                state.afk_end = Some(Local::now());
                let duration = state.afk_end.unwrap() - state.afk_start.unwrap();
                println!(
                    "✅ User returned at: {} (AFK for: {:?})",
                    state.afk_end.unwrap(),
                    duration
                );
    
                // 🛠 Reset AFK state
                state.is_afk = false;
                state.afk_start = None;
                state.afk_end = None;
            }
            state.last_activity = Instant::now();
        } 
        // 🚨 If idle time exceeds threshold, mark as AFK
        else if idle_time >= idle_threshold {
            if !state.is_afk {
                state.afk_start = Some(Local::now());
                state.is_afk = true;
                println!(
                    "🚨 AFK Triggered! Idle Time: {:?}, Threshold: {:?}, AFK State: {}",
                    idle_time, idle_threshold, state.is_afk
                );
            }
        } 
        // 🔄 If AFK but idle time resets, user has returned!
        else if state.is_afk && idle_time < idle_threshold {
            state.is_afk = false;
            state.afk_start = None;
            state.afk_end = None;
            println!("✅ User is back! Resetting AFK state.");
        }
    
        println!(
            "🕒 Idle time: {:?}, is_afk: {}",
            idle_time, state.is_afk
        );
    
        thread::sleep(Duration::from_secs(1));
    });
    }

#[command]
pub fn get_afk_status() -> AfkData {
    let state = AFK_STATE.lock().unwrap();

    println!(
        "📡 Fetching AFK Status: is_afk={} last_active={}s afk_start={:?} afk_end={:?}",
        state.is_afk,
        state.last_activity.elapsed().as_secs(),
        state.afk_start,
        state.afk_end
    );

    let afk_duration = if let (Some(start), Some(end)) = (state.afk_start, state.afk_end) {
        Some(format!("{:?}", end - start))
    } else {
        None
    };

    AfkData {
        last_active: state.last_activity.elapsed().as_secs().to_string(),
        afk_start: state.afk_start.map(|t| t.to_string()),
        afk_end: state.afk_end.map(|t| t.to_string()),
        afk_duration,
        is_afk: state.is_afk,
    }
}

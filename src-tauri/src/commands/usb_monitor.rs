use crate::commands::database::execute_write_query;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use rusqlite::ToSql;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use tauri::command;
use tokio::task;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::time::Duration;

/// **Creates the USB file transfers table if it doesn't exist**
fn init_usb_file_transfers_db() {
    let query = "
        CREATE TABLE IF NOT EXISTS usb_transfers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            filepath TEXT NOT NULL,
            transfer_type TEXT NOT NULL, -- 'device_to_usb' or 'usb_to_device'
            event_type TEXT NOT NULL, -- 'Created', 'Modified', 'Deleted', etc.
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ";
    execute_write_query(query, vec![]).expect("Failed to create usb_transfers table");
}


/// **Gets the USB mount path**
fn get_mount_path() -> Option<String> {
    let output = Command::new("wmic")
        .args(&["logicaldisk", "where", "DriveType=2", "get", "DeviceID"])
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let drive_letters: Vec<String> = output_str
        .lines()
        .skip(1)
        .filter_map(|line| {
            let drive = line.trim();
            if !drive.is_empty() {
                Some(format!("{}\\", drive)) // Ensuring correct path format (e.g., "E:\")
            } else {
                None
            }
        })
        .collect();

    drive_letters
        .into_iter()
        .find(|path| Path::new(path).exists())
}

lazy_static::lazy_static! {
    static ref RECENT_TRANSFERS: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
}

const DUPLICATE_EVENT_WINDOW: Duration = Duration::from_secs(2); // Ignore duplicate events within 2 seconds

#[command]
pub fn monitor_usb_file_transfers() {
    init_usb_file_transfers_db(); // Ensure table exists

    task::spawn_blocking(|| {
        let (tx, rx): (mpsc::Sender<Result<Event, notify::Error>>, Receiver<Result<Event, notify::Error>>) = mpsc::channel();
        let mut watcher = recommended_watcher(tx).expect("Failed to create watcher");

        if let Some(usb_path) = get_mount_path() {
            println!("Watching USB drive: {}", usb_path);
            watcher.watch(Path::new(&usb_path), RecursiveMode::Recursive)
                .expect("Failed to watch USB drive");
        } else {
            println!("No USB drive detected.");
            return;
        }

        for res in rx {
            match res {
                Ok(event) => {
                    if let Some(filepath) = event.paths.first() {
                        if let Some(filename) = filepath.file_name() {
                            let filename = filename.to_string_lossy().to_string();
                            let filepath_str = filepath.to_string_lossy().to_string(); // Convert to owned String

                            let usb_mount_path = get_mount_path().unwrap_or_default();
                            let transfer_type = if filepath_str.starts_with(&usb_mount_path) { 
                                "device_to_usb"
                            } else {
                                "usb_to_device"
                            };

                            // Determine event type
                            let event_type = match event.kind {
                                EventKind::Create(_) => "Created",
                                EventKind::Modify(_) => "Modified",
                                EventKind::Remove(_) => "Deleted",
                                _ => "Unknown",
                            };

                            let unique_key = format!("{}-{}-{}", filepath_str, transfer_type, event_type);

                            // **Debounce duplicate events**
                            let mut recent_transfers = RECENT_TRANSFERS.lock().unwrap();
                            if recent_transfers.contains(&unique_key) {
                                continue; // Skip duplicate event
                            }
                            
                            // Add to recent transfers set
                            recent_transfers.insert(unique_key.clone());

                            // Remove it after a short delay
                            let unique_key_clone = unique_key.clone();
                            std::thread::spawn(move || {
                                std::thread::sleep(DUPLICATE_EVENT_WINDOW);
                                RECENT_TRANSFERS.lock().unwrap().remove(&unique_key_clone);
                            });

                            // Insert into DB
                            let query = "
                                INSERT INTO usb_transfers (filename, filepath, transfer_type, event_type) 
                                VALUES (?, ?, ?, ?);
                            ";

                            let params: Vec<Box<dyn ToSql + Send + Sync>> = vec![
                                Box::new(filename),
                                Box::new(filepath_str.clone()),
                                Box::new(transfer_type),
                                Box::new(event_type),
                            ];

                            if let Err(err) = execute_write_query(query, params) {
                                eprintln!("❌ Failed to insert USB transfer: {}", err);
                            } else {
                                println!("✅ USB Transfer Recorded: [{}] {} -> {}", event_type, transfer_type, filepath_str);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("watch error: {:?}", e),
            }
        }
    });
}

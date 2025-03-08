use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
use std::sync::mpsc::{self, Receiver};
use std::path::Path;
use std::process::Command;
use tauri::command;
use tokio::task;

/// Function to get USB mount path
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

/// Starts USB file monitoring (Runs asynchronously)
#[command]
pub fn monitor_usb_file_transfers() {
    task::spawn_blocking(|| {
        let (tx, rx): (
            mpsc::Sender<Result<Event, notify::Error>>,
            Receiver<Result<Event, notify::Error>>,
        ) = mpsc::channel();
        let mut watcher = recommended_watcher(tx).expect("Failed to create watcher");

        if let Some(usb_path) = get_mount_path() {
            println!("Watching USB drive: {}", usb_path);
            watcher
                .watch(Path::new(&usb_path), RecursiveMode::Recursive)
                .expect("Failed to watch USB drive");
        } else {
            println!("No USB drive detected.");
            return;
        }

        for res in rx {
            match res {
                Ok(event) => println!("USB event: {:?}", event),
                Err(e) => eprintln!("watch error: {:?}", e),
            }
        }
    });
}

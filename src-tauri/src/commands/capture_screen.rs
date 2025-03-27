use screenshots::Screen;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use chrono::prelude::*; // For handling IST time
use image::{DynamicImage, ImageOutputFormat};
use tokio::time;
use tauri::command;
use std::time::Duration;

/// Directory to save screenshots
const SCREENSHOT_DIR: &str = "D:\\Meltx\\emsScreenshots";

/// Captures the current screen at full resolution and saves it.
#[command]
pub async fn get_capture_screen() -> Result<String, String> {
    // Ensure the directory exists
    let screenshot_path = Path::new(SCREENSHOT_DIR);
    if !screenshot_path.exists() {
        create_dir_all(screenshot_path).map_err(|e| e.to_string())?;
    }

    // Get all screens and select the primary one
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let screen = screens.get(0).ok_or("No screen found")?;
    
    // Capture the screen image at full resolution
    let image = screen.capture().map_err(|e| e.to_string())?;
    let img = DynamicImage::ImageRgba8(image.into());

    // Get current time in IST
    let now_utc = Utc::now();
    let now_ist = now_utc.with_timezone(&FixedOffset::east_opt(5 * 3600 + 1800).unwrap());
    let formatted_time = now_ist.format("%Y-%m-%d_%H-%M-%S").to_string();

    // Generate filename with IST timestamp
    let filename = format!("screenshot-{}.png", formatted_time);
    let filepath: PathBuf = screenshot_path.join(&filename);

    // Save as PNG (highest quality)
    let mut output_file = File::create(&filepath).map_err(|e| e.to_string())?;
    img.write_to(&mut output_file, ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(filepath.to_string_lossy().to_string())
}

/// Starts a background scheduler that captures high-resolution screenshots every 10 minutes.
pub async fn start_screenshot_scheduler() {
    tokio::spawn(async {
        let mut interval = time::interval(Duration::from_secs(600)); // 10 minutes
        loop {
            interval.tick().await;
            match get_capture_screen().await {
                Ok(filepath) => println!("High-resolution screenshot saved at: {}", filepath),
                Err(e) => eprintln!("Failed to capture screenshot: {}", e),
            }
        }
    });
}

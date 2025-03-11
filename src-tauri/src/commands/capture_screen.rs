use screenshots::Screen;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use image::{DynamicImage, ImageOutputFormat, imageops::FilterType};
use chrono::prelude::*; // For handling IST time
use tokio::time;
use tauri::command;

/// Directory to save screenshots
const SCREENSHOT_DIR: &str = "D:\\Meltx\\emsScreenshots";

/// Captures the current screen, compresses it, and saves the screenshot.
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
    
    // Capture the screen image
    let image = screen.capture().map_err(|e| e.to_string())?;
    let img = DynamicImage::ImageRgba8(image.into());

    // Resize the image to reduce size (scale down to 720x480)
    let resized_img = img.resize_exact(720, 480, FilterType::Lanczos3);

    // Get current time in IST
    let now_utc = Utc::now();
    let now_ist = now_utc.with_timezone(&FixedOffset::east_opt(5 * 3600 + 1800).unwrap());
    let formatted_time = now_ist.format("%Y-%m-%d_%H-%M-%S").to_string();

    // Generate filename with IST timestamp
    let filename = format!("screenshot-{}.jpg", formatted_time);
    let filepath: PathBuf = screenshot_path.join(&filename);

    // Compress and save as JPEG (Quality: 70%)
    let mut output_file = File::create(&filepath).map_err(|e| e.to_string())?;
    resized_img.write_to(&mut output_file, ImageOutputFormat::Jpeg(70)) // Adjust quality 40-50KB
        .map_err(|e| e.to_string())?;

    Ok(filepath.to_string_lossy().to_string())
}

/// Starts a background scheduler that captures compressed screenshots every 10 minutes.
pub async fn start_screenshot_scheduler() {
    tokio::spawn(async {
        let mut interval = time::interval(Duration::from_secs(600)); // 10 minutes
        loop {
            interval.tick().await;
            match get_capture_screen().await {
                Ok(filepath) => println!("Compressed screenshot saved at: {}", filepath),
                Err(e) => eprintln!("Failed to capture screenshot: {}", e),
            }
        }
    });
}

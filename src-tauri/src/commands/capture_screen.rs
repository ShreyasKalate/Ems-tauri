use screenshots::Screen;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use image::{DynamicImage, ImageOutputFormat};
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

    // Convert the raw screenshot to an ImageBuffer
    let img = DynamicImage::ImageRgba8(image.into());

    // Generate a filename with timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    
    let filename = format!("screenshot-{}.jpg", timestamp);
    let filepath: PathBuf = screenshot_path.join(&filename);

    // Compress and save as JPEG (Quality: 75%)
    let mut output_file = File::create(&filepath).map_err(|e| e.to_string())?;
    img.write_to(&mut output_file, ImageOutputFormat::Jpeg(5))
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

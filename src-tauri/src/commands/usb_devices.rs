use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tauri::command;

/// Struct for storing USB device information
#[derive(Serialize)]
pub struct UsbDevice {
    vendor_id: u16,
    product_id: u16,
    manufacturer: Option<String>,
    product: Option<String>,
    is_storage: bool,
    mount_path: Option<String>,
    files: Option<Vec<String>>, // List of files if storage device
}

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    is_dir: bool,
}

/// Gets a list of all connected USB devices and returns as JSON.
#[command]
pub fn list_usb_devices() -> Result<Vec<UsbDevice>, String> {
    let context = Context::new().map_err(|e| e.to_string())?;
    let mut devices_list = Vec::new();

    for device in context.devices().map_err(|e| e.to_string())?.iter() {
        if let Ok(usb_device) = get_device_info(&device) {
            devices_list.push(usb_device);
        }
    }

    Ok(devices_list)
}

/// Extracts detailed info from a USB device.
pub fn get_device_info<T: UsbContext>(device: &Device<T>) -> Result<UsbDevice, String> {
    let descriptor: DeviceDescriptor = device.device_descriptor().map_err(|e| e.to_string())?;
    let mut manufacturer = None;
    let mut product = None;
    let mut is_storage = false;
    let mut mount_path = None;
    let mut files = None;

    if let Ok(handle) = device.open() {
        let timeout = Duration::from_millis(100);

        if let Ok(lang) = handle.read_languages(timeout) {
            if !lang.is_empty() {
                if let Ok(manu) = handle.read_manufacturer_string(lang[0], &descriptor, timeout) {
                    manufacturer = Some(manu);
                }
                if let Ok(prod) = handle.read_product_string(lang[0], &descriptor, timeout) {
                    product = Some(prod);
                }
            }
        }
    }

    // Detect if the USB is a mass storage device dynamically
    is_storage = is_usb_storage_device(&device);

    if is_storage {
        // First, try to detect the USB mount path dynamically
        if let Some(mount) = get_mount_path() {
            mount_path = Some(mount.clone());
            files = Some(read_usb_files(&mount));
        } else if let Some(mount) = get_dynamic_usb_mount() {
            mount_path = Some(mount.clone());
            files = Some(read_usb_files(&mount));
        }
    }

    Ok(UsbDevice {
        vendor_id: descriptor.vendor_id(),
        product_id: descriptor.product_id(),
        manufacturer,
        product,
        is_storage,
        mount_path,
        files,
    })
}

/// **Checks if a USB device is a storage device based on its class**
fn is_usb_storage_device<T: UsbContext>(device: &Device<T>) -> bool {
    if let Ok(config) = device.active_config_descriptor() {
        for interface in config.interfaces() {
            for descriptor in interface.descriptors() {
                if descriptor.class_code() == 0x08 {
                    // 0x08 → USB Mass Storage Device Class
                    return true;
                }
            }
        }
    }
    false
}

/// Detects the newly inserted USB mount path dynamically
fn get_dynamic_usb_mount() -> Option<String> {
    let initial_drives = get_available_drives();
    thread::sleep(Duration::from_secs(3)); // Allow system to mount USB
    let new_drives = get_available_drives();

    // Find the new drive (USB)
    for drive in new_drives.difference(&initial_drives) {
        return Some(drive.clone());
    }

    None
}

/// Get a list of currently available drive letters
fn get_available_drives() -> HashSet<String> {
    let possible_paths = 'A'..='Z'; // Check all possible drive letters A-Z
    let mut drives = HashSet::new();

    for letter in possible_paths {
        let path = format!("{}:/", letter);
        if fs::metadata(&path).is_ok() {
            drives.insert(path);
        }
    }

    drives
}

/// Reads files and folders from the USB storage device
fn read_usb_files(mount_path: &str) -> Vec<String> {
    fs::read_dir(mount_path)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().into_string().unwrap_or_default())
                .collect()
        })
        .unwrap_or_else(|_| vec![]) // Return empty list if an error occurs
}

/// Detects the mounted path of the USB dynamically (Windows)
fn get_mount_path() -> Option<String> {
    let output = Command::new("wmic")
        .args(&["logicaldisk", "where", "DriveType=2", "get", "DeviceID"])
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let drive_letters: Vec<String> = output_str
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let drive = line.trim();
            if !drive.is_empty() {
                Some(format!("{}\\", drive))
            } else {
                None
            }
        })
        .collect();

    drive_letters.into_iter().find(|path| Path::new(path).exists())
}

/// Retrieves the contents of a folder inside the USB
#[tauri::command]
pub fn list_files_in_directory(path: String) -> Result<Vec<FileEntry>, String> {
    Ok(read_usb_files(&path)
        .into_iter()
        .map(|name| FileEntry { 
            name: name.clone(),  // Clone the name to prevent move error
            is_dir: Path::new(&name).is_dir() 
        })
        .collect())
}


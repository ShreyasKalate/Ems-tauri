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
    files: Option<Vec<FileEntry>>, // List of files if storage device
}

/// Struct for file entries (folders and files)
#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    is_dir: bool,
    files: Option<Vec<FileEntry>>, // Nested files if it's a directory
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

    is_storage = is_usb_storage_device(&device);

    if is_storage {
        if let Some(mount) = get_mount_path() {
            mount_path = Some(mount.clone());
            files = Some(list_files_recursive(&mount)?);
        } else if let Some(mount) = get_dynamic_usb_mount() {
            mount_path = Some(mount.clone());
            files = Some(list_files_recursive(&mount)?);
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
                    return true;
                }
            }
        }
    }
    false
}

/// Detects the USB mount path dynamically
fn get_dynamic_usb_mount() -> Option<String> {
    let initial_drives = get_available_drives();
    thread::sleep(Duration::from_secs(3));
    let new_drives = get_available_drives();

    for drive in new_drives.difference(&initial_drives) {
        return Some(drive.clone());
    }

    None
}

/// Get a list of available drive letters
fn get_available_drives() -> HashSet<String> {
    let mut drives = HashSet::new();

    for letter in 'A'..='Z' {
        let path = format!("{}:/", letter);
        if fs::metadata(&path).is_ok() {
            drives.insert(path);
        }
    }

    drives
}

/// Recursively fetches files inside folders and sorts them alphabetically
fn list_files_recursive(path: &str) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    let read_dir = fs::read_dir(path).map_err(|e| e.to_string())?;

    for entry in read_dir {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            let file_path = entry.path();
            let is_dir = file_path.is_dir();

            let mut file_entry = FileEntry {
                name: file_name.clone(),
                is_dir,
                files: None,
            };

            if is_dir {
                file_entry.files = Some(list_files_recursive(file_path.to_str().unwrap_or(""))?);
            }

            entries.push(file_entry);
        }
    }

    // Sort folders first, then files, both alphabetically
    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

/// Detects the USB mount path (Windows)
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
                Some(format!("{}\\", drive))
            } else {
                None
            }
        })
        .collect();

    drive_letters.into_iter().find(|path| Path::new(path).exists())
}

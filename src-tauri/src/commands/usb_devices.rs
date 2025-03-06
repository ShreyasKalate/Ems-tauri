use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{self, read_dir};
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

    // Check if device is a storage device (Pendrive)
    if is_pendrive(descriptor.vendor_id(), descriptor.product_id()) {
        is_storage = true;

        // First, try to detect the USB mount path dynamically
        if let Some(mount) = get_mount_path() {
            mount_path = Some(mount.clone());
            files = Some(read_usb_files(&mount)); // Read files from the correct path
        } else if let Some(mount) = get_dynamic_usb_mount() {
            mount_path = Some(mount.clone());
            files = Some(read_usb_files(&mount)); // Read files if detected
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

/// Function to check if a USB device is a pendrive (Modify for specific vendors)
fn is_pendrive(vendor_id: u16, _product_id: u16) -> bool {
    let storage_vendors = vec![0x0781, 0x090C, 0x0951, 0x13FE, 0x058F, 0x1A2C, 0x0930]; // SanDisk, Kingston, etc.
    storage_vendors.contains(&vendor_id)
}

/// Detects the newly inserted USB mount path dynamically
fn get_dynamic_usb_mount() -> Option<String> {
    let initial_drives = get_available_drives();

    // Wait for a few seconds to allow the system to mount the USB
    thread::sleep(Duration::from_secs(3));

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

/// Reads files from the USB storage device
fn read_usb_files(mount_path: &str) -> Vec<String> {
    fs::read_dir(mount_path)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.path().file_name()?.to_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|_| vec![]) // Return empty list if an error occurs
}

/// Function to detect the mounted path of the USB dynamically (Windows)
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

use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use serde::Serialize;
use std::time::Duration;
use tauri::command;

/// Struct for storing USB device information
#[derive(Serialize)]
pub struct UsbDevice {
    vendor_id: u16,
    product_id: u16,
    manufacturer: Option<String>,
    product: Option<String>,
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

    Ok(UsbDevice {
        vendor_id: descriptor.vendor_id(),
        product_id: descriptor.product_id(),
        manufacturer,
        product,
    })
}

use std::process::Command;
use std::net::UdpSocket;
use serde::Serialize;
use tauri::command;
use regex::Regex;

#[derive(Debug, Serialize)]
pub struct NetworkInfo {
    ssid: String,
    private_ip: String,
    public_ip: String,
    mac_address: String,
    network_type: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectedDevice {
    mac_address: String,
    ip_address: String,
    os_type: String,
    device_type: String,
}

/// ✅ Get Private IP Address
fn get_private_ip() -> String {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    socket
        .connect("8.8.8.8:80")
        .expect("Failed to connect to external server");
    let local_addr = socket.local_addr().expect("Failed to get local address");
    local_addr.ip().to_string()
}

/// ✅ Get Public IP Address
fn get_public_ip() -> String {
    let output = Command::new("curl")
        .arg("-s")
        .arg("https://ifconfig.me")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    output
}

/// ✅ Get MAC Address of the current system
fn get_mac_address() -> String {
    let output = Command::new("getmac")
        .arg("/FO")
        .arg("LIST")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let mac_regex = Regex::new(r"([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})").unwrap();
    mac_regex
        .find(&output)
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// ✅ Detect whether the system is connected via WiFi or Ethernet
fn get_network_type() -> String {
    let output = Command::new("ipconfig")
        .arg("/all")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    if output.contains("Wireless") {
        return "Wi-Fi".to_string();
    } else if output.contains("Ethernet") {
        return "Ethernet".to_string();
    }
    "Unknown".to_string()
}

/// ✅ Get WiFi SSID (Windows Only)
fn get_ssid() -> String {
    let output = Command::new("netsh")
        .args(&["wlan", "show", "interfaces"])
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let ssid_regex = Regex::new(r"SSID\s*:\s*(.*)").unwrap();

    if let Some(captures) = ssid_regex.captures(&output_str) {
        captures.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or("Unknown".to_string())
    } else {
        "Unknown".to_string()
    }
}

/// ✅ Fetch Network Information
#[command]
pub fn get_network_info() -> NetworkInfo {
    NetworkInfo {
        ssid: get_ssid(),
        private_ip: get_private_ip(),
        public_ip: get_public_ip(),
        mac_address: get_mac_address(),
        network_type: get_network_type(),
    }
}

/// ✅ Get List of Connected Devices via ARP
fn get_connected_devices() -> Vec<ConnectedDevice> {
    let output = Command::new("arp")
        .arg("-a")
        .output()
        .expect("Failed to execute arp command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    let mut devices = Vec::new();

    for line in lines.iter().skip(3) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let ip_address = parts[0].to_string();
            let mac_address = parts[1].to_string();
            let os_type = "Unknown".to_string(); 
            let device_type = "Unknown".to_string(); 

            devices.push(ConnectedDevice {
                mac_address,
                ip_address,
                os_type,
                device_type,
            });
        }
    }

    devices
}

/// ✅ Fetch Connected Devices
#[command]
pub fn get_connected_devices_list() -> Vec<ConnectedDevice> {
    get_connected_devices()
}

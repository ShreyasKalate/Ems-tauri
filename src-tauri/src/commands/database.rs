use rusqlite::Connection;
use once_cell::sync::Lazy;
use std::{sync::Mutex, time::Duration, thread};

use crate::commands::system;

// Shared DB connection for all modules
pub static DB_CONN: Lazy<Mutex<Connection>> = Lazy::new(|| {
    Mutex::new(Connection::open("ems_data.db").expect("Failed to open database"))
});

/// Step 1: Create all tables
pub fn create_tables() {
    system::create_ram_usage_table();
}

/// Step 2: Start sync loop for all modules
pub fn start_db_operations() {
    loop {
        system::ram_usage_db(); // write from cache to SQLite
    }
}

/// Final cleanup on exit
pub fn clean_exit() {
    drop(DB_CONN.lock().unwrap_or_else(|e| e.into_inner()));
    println!("✅ DB connection closed cleanly.");
}

use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref DB_CONN: Arc<Mutex<Connection>> =
        Arc::new(Mutex::new(Connection::open("ems_data.db").expect("Failed to open database")));
}

/// Executes a write query in a single-threaded manner.
pub fn execute_write_query(query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<()> {
    let conn = DB_CONN.lock().unwrap();
    
    // Print query only (since params are not Debug)
    println!("Executing query: {}", query);

    let mut stmt = conn.prepare(query)?;
    stmt.execute(params)?;
    Ok(())
}

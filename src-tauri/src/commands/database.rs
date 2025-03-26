use crossbeam_channel::{unbounded, Receiver, Sender};
use rusqlite::{Connection, Result, ToSql};
use std::sync::{Arc, Mutex};
use std::thread;

lazy_static::lazy_static! {
    static ref DB_CONN: Arc<Mutex<Connection>> =
        Arc::new(Mutex::new(Connection::open("ems_data.db").expect("Failed to open database")));
}

pub struct DbWriteRequest {
    query: String,
    params: Vec<Box<dyn ToSql + Send + Sync>>, // Store params as Boxed ToSql
    result_sender: Option<std::sync::mpsc::Sender<i64>>,  // Add this field
}

lazy_static::lazy_static! {
    pub static ref DB_WRITE_SENDER: Sender<DbWriteRequest> = {
        let (sender, receiver): (Sender<DbWriteRequest>, Receiver<DbWriteRequest>) = unbounded();
        start_db_writer(receiver);
        sender
    };
}

/// **Starts a background thread for executing queued database writes**
fn start_db_writer(receiver: Receiver<DbWriteRequest>) {
    thread::spawn(move || {
        let conn = DB_CONN.lock().expect("Failed to lock database connection");

        // ✅ Enable WAL mode to prevent full database locks
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .expect("❌ Failed to enable WAL mode");

        for request in receiver.iter() {
            let param_refs: Vec<&dyn ToSql> = request
                .params
                .iter()
                .map(|p| p.as_ref() as &dyn ToSql)
                .collect();

            let mut retries = 5; // ✅ Retry up to 5 times if locked

            while retries > 0 {
                match conn.execute(&request.query, &param_refs[..]) {
                    Ok(_) => {
                        if request.query.trim_start().to_uppercase().starts_with("INSERT") {
                            if let Ok(id) = conn.last_insert_rowid().try_into() {
                                if let Some(sender) = request.result_sender {
                                    let _ = sender.send(id);
                                }
                            }
                        } else {
                            if let Some(sender) = request.result_sender {
                                let _ = sender.send(0);
                            }
                        }
                        break; // ✅ Break if successful
                    }
                    Err(err) => {
                        if err.to_string().contains("database is locked") {
                            eprintln!("🔴 Database is locked. Retrying... ({}/5)", 6 - retries);
                            retries -= 1;
                            std::thread::sleep(std::time::Duration::from_millis(50)); // ✅ Short delay before retry
                        } else {
                            eprintln!("❌ Failed to execute query: {}\n🔴 Error: {}", request.query, err);
                            if let Some(sender) = request.result_sender {
                                let _ = sender.send(-1);
                            }
                            break; // ✅ Stop retrying on other errors
                        }
                    }
                }
            }
        }
    });
}

pub fn execute_read_query<T, F>(
    query: &str,
    params: Vec<Box<dyn ToSql + Send + Sync>>,
    map_fn: F,
) -> Result<Vec<T>, String>
where
    F: Fn(&rusqlite::Row) -> rusqlite::Result<T>,
{
    let conn = DB_CONN.lock().expect("Failed to lock database connection");
    let mut stmt = match conn.prepare(query) {
        Ok(stmt) => stmt,
        Err(err) => return Err(format!("❌ Failed to prepare query: {}", err)),
    };

    // ✅ Explicitly remove `Send + Sync` trait bound from references
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| &**p as &dyn ToSql).collect();

    let rows = match stmt.query_map(param_refs.as_slice(), map_fn) {
        Ok(rows) => rows,
        Err(err) => return Err(format!("❌ Failed to execute query: {}", err)),
    };

    let mut results = Vec::new();
    for row in rows {
        match row {
            Ok(value) => results.push(value),
            Err(err) => eprintln!("❌ Error mapping row: {}", err),
        }
    }

    Ok(results)
}


pub fn execute_write_query(query: &str, params: Vec<Box<dyn ToSql + Send + Sync>>) -> Result<i64, String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    
    if let Err(err) = DB_WRITE_SENDER.send(DbWriteRequest {
        query: query.to_string(),
        params,
        result_sender: Some(sender),
    }) {
        return Err(format!("❌ Failed to send query to queue: {}", err));
    }

    match receiver.recv() {
        Ok(id) => Ok(id),
        Err(err) => Err(format!("❌ Failed to receive query result: {}", err)),
    }
}


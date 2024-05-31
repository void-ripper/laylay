use std::{error::Error, path::PathBuf};

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::errors::ServerErrors;


pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(folder: PathBuf) -> Result<Self, Box<dyn Error>> {
        let filename = folder.join("laylay.db");
        let existed = filename.exists();
        let conn = Connection::open(filename)?;

        if !existed {
            conn.execute_batch(include_str!("schema.sql"))?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub async fn save_log(&self, lvl: &str, target: &str, msg: &str) -> Result<(), ServerErrors> {
        let sql = r#"
        INSERT INTO logs(level_id, target, message) 
        VALUES((SELECT id FROM log_level WHERE name = ?), ?, ?)
        "#;
        let conn = self.conn.lock().await;
        let mut stmnt = conn.prepare_cached(sql)?;
        stmnt.execute((lvl, target, msg))?;

        Ok(())
    }
}
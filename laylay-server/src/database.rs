use std::{error::Error, path::PathBuf};

use laylay_common::{Bytes, Info, Version};
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

    pub async fn get_session_id(
        &self,
        pubkey: &Bytes,
        version: &Version,
        info: &Info,
    ) -> Result<i64, ServerErrors> {
        let pubhex = hex::encode(pubkey);
        let user_sql = r#"
            INSERT INTO user(pubkey)
            VALUES(?)
            ON CONFLICT (pubkey) DO NOTHING
            RETURNING id
        "#;
        let version_sql = r#"
            INSERT INTO version(major, minor, patch, target)
            VALUES(?, ?, ?, ?)
            ON CONFLICT (major, minor, patch, target) DO NOTHING
            RETURNING id
        "#;
        let info_sql = r#"
            INSERT INTO sysinfo(
                name,            
                host_name,            
                kernel_version,            
                os_version,            
                cpu_name,            
                cpu_vendor,            
                cpu_brand,            
                cpu_freq,            
                memory
            )
            VALUES(?,?,?,?,?,?,?,?,?)
            ON CONFLICT (
                name,            
                host_name,            
                kernel_version,            
                os_version,            
                cpu_name,            
                cpu_vendor,            
                cpu_brand,            
                cpu_freq,            
                memory
            ) DO NOTHING
            RETURNING id
        "#;
        let uvs_sql = r#"
            INSERT INTO user_version_sys(user_id, version_id, sysinfo_id)
            VALUES (?, ?, ?)
            ON CONFLICT (user_id, version_id, sysinfo_id) DO NOTHING
            RETURNING id
        "#;
        let session_sql = r#"
            INSERT INTO  user_session(uvs_id, started)
            VALUES(?, datetime())
            RETURNING id
        "#;
        let conn = self.conn.lock().await;
        let mut user_stmnt = conn.prepare_cached(user_sql)?;
        let mut version_stmnt = conn.prepare_cached(version_sql)?;
        let mut info_stmnt = conn.prepare_cached(info_sql)?;
        let mut uvs_stmnt = conn.prepare_cached(uvs_sql)?;
        let mut session_stmnt = conn.prepare_cached(session_sql)?;

        let user_id: i64 = user_stmnt.query_row((pubhex,), |r| r.get(0))?;
        let version_id: i64 = version_stmnt.query_row(
            (
                &version.major,
                &version.minor,
                &version.patch,
                &version.target,
            ),
            |r| r.get(0),
        )?;
        let info_id: i64 = info_stmnt.query_row(
            (
                &info.name,
                &info.host_name,
                &info.kernel_version,
                &info.os_version,
                &info.cpu.name,
                &info.cpu.vendor_id,
                &info.cpu.brand,
                &info.cpu.freq,
                &info.memory,
            ),
            |r| r.get(0),
        )?;
        let uvs_id: i64 = uvs_stmnt.query_row((user_id, version_id, info_id), |r| r.get(0))?;
        let session_id: i64 = session_stmnt.query_row((uvs_id,), |r| r.get(0))?;

        Ok(session_id)
    }

    pub async fn end_session(&self, session_id: i64) -> Result<(), ServerErrors> {
        let sql = r#"
            UPDATE user_session SET ended = datetime() WHERE id = ?
        "#;
        let conn = self.conn.lock().await;
        let mut stmnt = conn.prepare_cached(sql)?;
        stmnt.execute((session_id,))?;

        Ok(())
    }
}

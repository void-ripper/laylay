use std::{error::Error, path::PathBuf};

use laylay_common::{Bytes, Info, Version};
use rusqlite::{Connection, OptionalExtension};
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

        let select_user_sql = r#"SELECT id FROM user WHERE pubkey = ?"#;
        let insert_user_sql = r#"
            INSERT INTO user(pubkey)
            VALUES(?)
            RETURNING id
        "#;
        let select_version_sql =
            r#"SELECT id FROM version WHERE major = ? AND minor = ? AND patch = ? AND  target = ?"#;
        let insert_version_sql = r#"
            INSERT INTO version(major, minor, patch, target)
            VALUES(?, ?, ?, ?)
            RETURNING id
        "#;
        let select_info_sql = r#"
        SELECT id FROM sysinfo WHERE
            name = ?            
            AND host_name = ?            
            AND kernel_version = ?            
            AND os_version = ?            
            AND cpu_name = ?            
            AND cpu_vendor = ?            
            AND cpu_brand = ?            
            AND cpu_freq = ?            
            AND memory = ?
        "#;
        let insert_info_sql = r#"
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
            RETURNING id
        "#;
        let select_uvs_sql = r#"
            SELECT id FROM user_version_sys
            WHERE user_id = ? AND version_id = ? AND sysinfo_id = ?
        "#;
        let insert_uvs_sql = r#"
            INSERT INTO user_version_sys(user_id, version_id, sysinfo_id)
            VALUES (?, ?, ?)
            RETURNING id
        "#;
        let session_sql = r#"
            INSERT INTO  user_session(uvs_id, started)
            VALUES(?, datetime())
            RETURNING id
        "#;
        let conn = self.conn.lock().await;
        let mut user_stmnt = conn.prepare_cached(select_user_sql)?;
        let mut version_stmnt = conn.prepare_cached(select_version_sql)?;
        let mut info_stmnt = conn.prepare_cached(select_info_sql)?;
        let mut uvs_stmnt = conn.prepare_cached(select_uvs_sql)?;
        let mut session_stmnt = conn.prepare_cached(session_sql)?;

        let user_id: Option<i64> = user_stmnt
            .query_row((&pubhex,), |r| r.get(0))
            .optional()
            .map_err(|e| ServerErrors::db(e, "get user id"))?;
        let user_id: i64 = if let Some(user_id) = user_id {
            user_id
        } else {
            let mut stmnt = conn.prepare_cached(&insert_user_sql)?;
            stmnt.query_row((pubhex,), |r| r.get(0))?
        };

        let version_params = (
            &version.major,
            &version.minor,
            &version.patch,
            &version.target,
        );
        let version_id: Option<i64> = version_stmnt
            .query_row(version_params, |r| r.get(0))
            .optional()
            .map_err(|e| ServerErrors::db(e, "get version"))?;
        let version_id: i64 = if let Some(version_id) = version_id {
            version_id
        } else {
            let mut stmnt = conn.prepare_cached(&insert_version_sql)?;
            stmnt.query_row(version_params, |r| r.get(0))?
        };

        let info_params = (
            &info.name,
            &info.host_name,
            &info.kernel_version,
            &info.os_version,
            &info.cpu.name,
            &info.cpu.vendor_id,
            &info.cpu.brand,
            &info.cpu.freq,
            &info.memory,
        );
        let info_id: Option<i64> = info_stmnt
            .query_row(info_params, |r| r.get(0))
            .optional()
            .map_err(|e| ServerErrors::db(e, "get info"))?;
        let info_id: i64 = if let Some(id) = info_id {
            id
        } else {
            let mut stmnt = conn.prepare_cached(&insert_info_sql)?;
            stmnt.query_row(info_params, |r| r.get(0))?
        };

        let uvs_params = (user_id, version_id, info_id);
        let uvs_id: Option<i64> = uvs_stmnt
            .query_row(uvs_params, |r| r.get(0))
            .optional()
            .map_err(|e| ServerErrors::db(e, "get use version info id"))?;
        let uvs_id: i64 = if let Some(id) = uvs_id {
            id
        } else {
            let mut stmnt = conn.prepare_cached(&insert_uvs_sql)?;
            stmnt.query_row(uvs_params, |r| r.get(0))?
        };

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

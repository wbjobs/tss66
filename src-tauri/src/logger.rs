use crate::config::AppConfig;
use crate::guardian::KilledProcessInfo;
use crate::ring_buffer::ProcessMetric;
use chrono::Local;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricLog {
    pub id: i64,
    pub timestamp: String,
    pub process_name: String,
    pub pid: Option<u32>,
    pub running: bool,
    pub cpu_usage: f64,
    pub memory_mb: f64,
    pub threshold_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub process_name: String,
    pub detail: String,
}

pub struct Logger {
    conn: Connection,
}

impl Logger {
    pub fn new() -> Result<Self, String> {
        let path = Self::db_path()?;
        let conn = Connection::open(&path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS metric_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                process_name TEXT NOT NULL,
                pid INTEGER,
                running INTEGER NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_mb REAL NOT NULL,
                threshold_mb REAL
            );

            CREATE INDEX IF NOT EXISTS idx_metric_logs_timestamp 
                ON metric_logs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_metric_logs_process 
                ON metric_logs(process_name);

            CREATE TABLE IF NOT EXISTS event_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                process_name TEXT NOT NULL,
                detail TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_event_logs_timestamp 
                ON event_logs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_event_logs_type 
                ON event_logs(event_type);
            "#,
        )
        .map_err(|e| format!("初始化数据库失败: {}", e))?;

        // 开启 WAL 模式和缓存优化
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            "#,
        )
        .map_err(|e| format!("设置数据库参数失败: {}", e))?;

        Ok(Self { conn })
    }

    fn db_path() -> Result<PathBuf, String> {
        let dir = AppConfig::data_dir()?;
        Ok(dir.join("process_logs.db3"))
    }

    pub fn insert_metrics(&mut self, metrics: &[ProcessMetric]) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let tx = self.conn.unchecked_transaction();
            if let Err(e) = tx {
                eprintln!("开始数据库事务失败: {}", e);
                return;
            }
            let tx = tx.unwrap();
            {
                let mut stmt = match tx.prepare(
                    "INSERT INTO metric_logs (timestamp, process_name, pid, running, cpu_usage, memory_mb, threshold_mb)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("准备 metric 语句失败: {}", e);
                        return;
                    }
                };

                for m in metrics {
                    if let Err(e) = stmt.execute(params![
                        now,
                        m.name,
                        m.pid,
                        m.running as i32,
                        m.cpu_usage,
                        m.memory_mb,
                        m.threshold_mb,
                    ]) {
                        eprintln!("写入 metric 日志失败: {}", e);
                    }
                }
            }
            if let Err(e) = tx.commit() {
                eprintln!("提交事务失败: {}", e);
            }
        }));

        if let Err(e) = result {
            eprintln!("日志写入发生 panic: {:?}", e);
        }
    }

    pub fn insert_event(&mut self, event_type: &str, process_name: &str, detail: &str) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = self.conn.execute(
            "INSERT INTO event_logs (timestamp, event_type, process_name, detail)
             VALUES (?1, ?2, ?3, ?4)",
            params![now, event_type, process_name, detail],
        );
        if let Err(e) = result {
            eprintln!("写入事件日志失败: {}", e);
        }
    }

    pub fn log_killed(&mut self, info: &KilledProcessInfo) {
        let detail = format!(
            "内存占用 {:.2}MB 超过阈值 {:.2}MB，已自动终止",
            info.memory_mb, info.threshold_mb
        );
        self.insert_event("PROCESS_KILLED", &info.name, &detail);
    }

    pub fn log_launched(&mut self, process_name: &str, path: &str) {
        let detail = format!("启动程序: {}", path);
        self.insert_event("PROCESS_LAUNCHED", process_name, &detail);
    }

    pub fn query_metric_logs(
        &self,
        process_name: Option<&str>,
        limit: i64,
    ) -> Result<Vec<MetricLog>, String> {
        let sql;
        let params: Vec<Box<dyn rusqlite::ToSql>>;

        if let Some(name) = process_name {
            sql = "SELECT id, timestamp, process_name, pid, running, cpu_usage, memory_mb, threshold_mb
                   FROM metric_logs WHERE process_name = ?1 ORDER BY id DESC LIMIT ?2";
            params = vec![Box::new(name.to_string()), Box::new(limit)];
        } else {
            sql = "SELECT id, timestamp, process_name, pid, running, cpu_usage, memory_mb, threshold_mb
                   FROM metric_logs ORDER BY id DESC LIMIT ?1";
            params = vec![Box::new(limit)];
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| format!("准备查询语句失败: {}", e))?;

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(MetricLog {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    process_name: row.get(2)?,
                    pid: row.get(3)?,
                    running: row.get::<_, i32>(4)? != 0,
                    cpu_usage: row.get(5)?,
                    memory_mb: row.get(6)?,
                    threshold_mb: row.get(7)?,
                })
            })
            .map_err(|e| format!("执行查询失败: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            match row {
                Ok(r) => result.push(r),
                Err(e) => eprintln!("解析日志行失败: {}", e),
            }
        }
        Ok(result)
    }

    pub fn query_event_logs(
        &self,
        event_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<EventLog>, String> {
        let sql;
        let params: Vec<Box<dyn rusqlite::ToSql>>;

        if let Some(et) = event_type {
            sql = "SELECT id, timestamp, event_type, process_name, detail
                   FROM event_logs WHERE event_type = ?1 ORDER BY id DESC LIMIT ?2";
            params = vec![Box::new(et.to_string()), Box::new(limit)];
        } else {
            sql = "SELECT id, timestamp, event_type, process_name, detail
                   FROM event_logs ORDER BY id DESC LIMIT ?1";
            params = vec![Box::new(limit)];
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| format!("准备查询语句失败: {}", e))?;

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(EventLog {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    event_type: row.get(2)?,
                    process_name: row.get(3)?,
                    detail: row.get(4)?,
                })
            })
            .map_err(|e| format!("执行查询失败: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            match row {
                Ok(r) => result.push(r),
                Err(e) => eprintln!("解析日志行失败: {}", e),
            }
        }
        Ok(result)
    }

    pub fn cleanup_old_logs(&mut self, days_keep: u32) {
        let sql = format!(
            "DELETE FROM metric_logs WHERE timestamp < datetime('now', '-{} day')",
            days_keep
        );
        if let Err(e) = self.conn.execute(&sql, []) {
            eprintln!("清理旧 metric 日志失败: {}", e);
        }
        let sql2 = format!(
            "DELETE FROM event_logs WHERE timestamp < datetime('now', '-{} day')",
            days_keep
        );
        if let Err(e) = self.conn.execute(&sql2, []) {
            eprintln!("清理旧 event 日志失败: {}", e);
        }
        let _ = self.conn.execute("VACUUM", []);
    }
}

pub type SharedLogger = Arc<Mutex<Logger>>;

use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;
use voxflow_config::data_dir;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("database: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("config: {0}")]
    Config(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub duration_raw_secs: f32,
    pub duration_billable_secs: f32,
    pub active_app_id: Option<String>,
    pub estimated_usd: f32,
}

/// Thread-safe history store — opens SQLite per operation.
#[derive(Debug, Default, Clone)]
pub struct HistoryStore;

impl HistoryStore {
    pub fn open() -> Result<Self, HistoryError> {
        let store = Self;
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), HistoryError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                duration_raw_secs REAL NOT NULL,
                duration_billable_secs REAL NOT NULL,
                active_app_id TEXT,
                estimated_usd REAL NOT NULL
            );
            CREATE TABLE IF NOT EXISTS usage_monthly (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                month INTEGER NOT NULL,
                year INTEGER NOT NULL,
                data_json TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    fn connect(&self) -> Result<Connection, HistoryError> {
        let path = db_path().ok_or_else(|| HistoryError::Config("data dir unavailable".into()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Connection::open(path)?)
    }

    pub fn insert(&self, entry: &HistoryEntry) -> Result<(), HistoryError> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO history (id, text, created_at, provider, model, duration_raw_secs, duration_billable_secs, active_app_id, estimated_usd)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.id,
                entry.text,
                entry.created_at.to_rfc3339(),
                entry.provider,
                entry.model,
                entry.duration_raw_secs,
                entry.duration_billable_secs,
                entry.active_app_id,
                entry.estimated_usd,
            ],
        )?;
        Ok(())
    }

    pub fn list(&self, limit: u32) -> Result<Vec<HistoryEntry>, HistoryError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, text, created_at, provider, model, duration_raw_secs, duration_billable_secs, active_app_id, estimated_usd
             FROM history ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                text: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                provider: row.get(3)?,
                model: row.get(4)?,
                duration_raw_secs: row.get(5)?,
                duration_billable_secs: row.get(6)?,
                active_app_id: row.get(7)?,
                estimated_usd: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(HistoryError::from)
    }

    pub fn delete_older_than_days(&self, days: u32) -> Result<u64, HistoryError> {
        let conn = self.connect()?;
        let cutoff = Local::now() - chrono::Duration::days(days as i64);
        let count = conn.execute(
            "DELETE FROM history WHERE created_at < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        Ok(count as u64)
    }

    pub fn enforce_limit(&self, max_entries: u32) -> Result<(), HistoryError> {
        if max_entries == u32::MAX {
            return Ok(());
        }
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM history WHERE id NOT IN (
                SELECT id FROM history ORDER BY created_at DESC LIMIT ?1
            )",
            params![max_entries],
        )?;
        Ok(())
    }

    pub fn export_json(&self, limit: u32) -> Result<String, HistoryError> {
        let entries = self.list(limit)?;
        Ok(serde_json::to_string_pretty(&entries)?)
    }

    pub fn export_csv(&self, limit: u32) -> Result<String, HistoryError> {
        let entries = self.list(limit)?;
        let mut csv = String::from("id,created_at,provider,model,duration_secs,text\n");
        for e in entries {
            let text = e.text.replace('"', "\"\"");
            csv.push_str(&format!(
                "{},{},{},{},{:.2},\"{}\"\n",
                e.id,
                e.created_at.to_rfc3339(),
                e.provider,
                e.model,
                e.duration_billable_secs,
                text
            ));
        }
        Ok(csv)
    }

    pub fn save_monthly_usage(
        &self,
        month: u32,
        year: i32,
        json: &str,
    ) -> Result<(), HistoryError> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM usage_monthly WHERE month = ?1 AND year = ?2",
            params![month, year],
        )?;
        conn.execute(
            "INSERT INTO usage_monthly (month, year, data_json) VALUES (?1, ?2, ?3)",
            params![month, year, json],
        )?;
        Ok(())
    }

    pub fn load_monthly_usage(
        &self,
        month: u32,
        year: i32,
    ) -> Result<Option<String>, HistoryError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT data_json FROM usage_monthly WHERE month = ?1 AND year = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![month, year])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

pub fn new_entry(
    text: String,
    provider: &str,
    model: &str,
    duration_raw: f32,
    duration_billable: f32,
    active_app_id: Option<String>,
    estimated_usd: f32,
) -> HistoryEntry {
    HistoryEntry {
        id: Uuid::new_v4().to_string(),
        text,
        created_at: Utc::now(),
        provider: provider.into(),
        model: model.into(),
        duration_raw_secs: duration_raw,
        duration_billable_secs: duration_billable,
        active_app_id,
        estimated_usd,
    }
}

fn db_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("voxflow.db"))
}

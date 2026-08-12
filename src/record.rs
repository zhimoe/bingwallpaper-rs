use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub url: String,
    pub local_file: String,
    pub description: String,
    pub time: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Vec<u8>>,
    pub is_accompany: bool,
    pub market: String,
}

impl DownloadRecord {
    pub fn new(
        url: String,
        local_file: String,
        description: String,
        active_time: (Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>),
        raw: Option<Vec<u8>>,
        is_accompany: bool,
        market: String,
    ) -> Self {
        let (start_time, end_time) = active_time;
        let now = Utc::now().to_rfc3339();
        DownloadRecord {
            url,
            local_file,
            description,
            time: now,
            start_time: start_time.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            end_time: end_time.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            raw,
            is_accompany,
            market,
        }
    }
}

#[derive(Debug, Default)]
pub struct DownloadRecordManager {
    records: HashMap<String, DownloadRecord>,
}

impl DownloadRecordManager {
    pub fn new() -> Self {
        DownloadRecordManager {
            records: HashMap::new(),
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(&mut self, path: &str) {
        self.records.clear();
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    log::info!("{} not found, ignore download history", path);
                } else {
                    log::warn!("error occurs when recover downloading history: {}", e);
                }
                return;
            }
        };
        let content: HashMap<String, DownloadRecord> = match serde_json::from_str(&data) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("error occurs when load json file: {}", e);
                return;
            }
        };
        log::debug!("json file loaded: {:?}", content);
        for r in content.into_values() {
            if Path::new(&r.local_file).is_file() {
                self.add(r);
            } else {
                log::debug!("{} doesn't exist any more", r.local_file);
            }
        }
        log::debug!("history loaded");
    }

    pub fn add(&mut self, r: DownloadRecord) {
        let mut record = r;
        record.raw = None;
        self.records.insert(record.url.clone(), record);
    }

    pub fn get_by_url(&self, url: &str) -> Option<&DownloadRecord> {
        self.records.get(url)
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

pub struct SqlDatabaseRecordManager {
    records: HashMap<String, DownloadRecord>,
}

impl SqlDatabaseRecordManager {
    pub const LATEST_DB_VERSION: (i32, i32, i32) = (5, 6, 1);

    pub fn new() -> Self {
        SqlDatabaseRecordManager {
            records: HashMap::new(),
        }
    }

    pub fn add(&mut self, r: DownloadRecord) {
        self.records.insert(r.url.clone(), r);
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("trying to save history to {}", path);
        let mut conn = Connection::open(path)?;
        let transaction = conn.transaction()?;
        self.upgrade_db(&transaction)?;
        for (k, v) in &self.records {
            transaction.execute(
                "INSERT OR REPLACE INTO [BingWallpaperRecords]
                  (Url, DownloadTime, StartTime, EndTime, LocalFilePath, Description, Image, IsAccompany, Market)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    k,
                    v.time,
                    v.start_time,
                    v.end_time,
                    v.local_file,
                    v.description,
                    v.raw.as_ref(),
                    v.is_accompany,
                    v.market
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn upgrade_db(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let ver = self.judge_version(conn)?;
        log::debug!("dealing with database created in {:?}", ver);
        if ver == (0, 0, 0) {
            self.create_scheme(conn)?;
            return Ok(());
        } else if self.vercmp(ver, Self::LATEST_DB_VERSION) > 0 {
            return Err(format!(
                "Can't deal with database created by higher program version {:?}",
                ver
            )
            .into());
        } else if ver == Self::LATEST_DB_VERSION {
            return Ok(());
        }
        log::info!(
            "current db version {:?} needs upgrade to {:?}",
            ver,
            Self::LATEST_DB_VERSION
        );
        let mut current_ver = ver;
        while self.vercmp(current_ver, Self::LATEST_DB_VERSION) < 0 {
            let (next_ver, script) = match current_ver {
                (4, 4, 1) => (
                    (4, 4, 2),
                    r#"ALTER TABLE [BingWallpaperRecords]
ADD COLUMN Market TEXT(64) DEFAULT "";

CREATE TABLE [BingWallpaperCore]
([MajorVer] INTEGER,
  [MinorVer] INTEGER,
  [Build] INTEGER);

INSERT INTO [BingWallpaperCore]
  (MajorVer, MinorVer, Build)
  VALUES (4, 4, 2);"#,
                ),
                (4, 4, 2) => (
                    (5, 6, 1),
                    r#"ALTER TABLE [BingWallpaperRecords]
ADD COLUMN StartTime DATETIME DEFAULT NULL;
ALTER TABLE [BingWallpaperRecords]
ADD COLUMN EndTime DATETIME DEFAULT NULL;

UPDATE [BingWallpaperCore]
  SET MajorVer=5, MinorVer=6, Build=1
  WHERE MajorVer=4 AND MinorVer=4 AND Build=2;"#,
                ),
                _ => return Err(format!("Unknown version {:?}", current_ver).into()),
            };
            log::debug!(
                "upgrading database from {:?} to {:?}, execute script",
                current_ver,
                next_ver
            );
            conn.execute_batch(script)?;
            current_ver = next_ver;
        }
        log::info!(
            "upgrading script executed successfully, now db is {:?}",
            self.judge_version(conn)?
        );
        Ok(())
    }

    fn create_scheme(&self, conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE [BingWallpaperRecords] (
              [Url] CHAR(1024) NOT NULL ON CONFLICT FAIL,
              [DownloadTime] DATETIME NOT NULL ON CONFLICT FAIL,
              [StartTime] DATETIME,
              [EndTime] DATETIME,
              [LocalFilePath] CHAR(1024),
              [Description] TEXT(1024),
              [Market] TEXT(64) DEFAULT \"\",
              [Image] BLOB,
              [IsAccompany] BOOLEAN DEFAULT False,
              CONSTRAINT [sqlite_autoindex_BingWallpaperRecords_1] PRIMARY KEY ([Url]));",
            [],
        )?;
        conn.execute(
            "CREATE TABLE [BingWallpaperCore] (
              [MajorVer] INTEGER,
              [MinorVer] INTEGER,
              [Build] INTEGER);",
            [],
        )?;
        conn.execute(
            "INSERT INTO [BingWallpaperCore]
              (MajorVer, MinorVer, Build)
              VALUES (?1, ?2, ?3)",
            rusqlite::params![5, 6, 1],
        )?;
        log::debug!(
            "created db from prog version {:?}",
            self.judge_version(conn)
        );
        Ok(())
    }

    fn vercmp(&self, ver1: (i32, i32, i32), ver2: (i32, i32, i32)) -> i32 {
        match ver1.cmp(&ver2) {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
        }
    }

    fn judge_version(
        &self,
        conn: &Connection,
    ) -> Result<(i32, i32, i32), Box<dyn std::error::Error>> {
        let mut stmt =
            conn.prepare("SELECT lower(name) FROM [sqlite_master] WHERE type=='table';")?;
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        log::debug!("find tables {:?} in database", tables);
        let ver = if !tables.contains(&"bingwallpaperrecords".to_string())
            && !tables.contains(&"bingwallpapercore".to_string())
        {
            (0, 0, 0)
        } else if tables.contains(&"bingwallpaperrecords".to_string())
            && !tables.contains(&"bingwallpapercore".to_string())
        {
            (4, 4, 1)
        } else {
            let mut stmt =
                conn.prepare("SELECT MajorVer, MinorVer, Build FROM [BingWallpaperCore];")?;
            stmt.query_row([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                ))
            })
            .optional()?
            .unwrap_or((0, 0, 0))
        };
        Ok(ver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_latest_database_schema() {
        let conn = Connection::open_in_memory().unwrap();
        let manager = SqlDatabaseRecordManager::new();

        manager.upgrade_db(&conn).unwrap();

        assert_eq!(
            manager.judge_version(&conn).unwrap(),
            SqlDatabaseRecordManager::LATEST_DB_VERSION
        );
    }

    #[test]
    fn upgrades_legacy_database_schema() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE BingWallpaperRecords (
                Url TEXT PRIMARY KEY,
                DownloadTime DATETIME NOT NULL,
                LocalFilePath TEXT,
                Description TEXT,
                Image BLOB,
                IsAccompany BOOLEAN DEFAULT False
            );",
        )
        .unwrap();
        let manager = SqlDatabaseRecordManager::new();

        manager.upgrade_db(&conn).unwrap();

        assert_eq!(
            manager.judge_version(&conn).unwrap(),
            SqlDatabaseRecordManager::LATEST_DB_VERSION
        );
    }
}

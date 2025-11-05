use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::SystemTime;
use bitflags::bitflags;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, Pool, Sqlite};
use sqlx::types::chrono;
use sqlx::types::chrono::Utc;
use tauri::async_runtime::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkshopItem {
    publishedfileid: String,
    title: String
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddonFlags(u32);
bitflags! {
    impl AddonFlags: u32 {
        /// Is addon in the 'workshop' folder
        const Workshop = 0b0001;
        /// Is addon a campaign
        const Campaign = 0b0010;
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AddonEntry {
    pub filename: String,
    pub updated_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub file_size: i64,
    pub workshop_info: Option<WorkshopItem>,
    pub flags: AddonFlags,
    #[serde(default)]
    pub tags: Vec<String>
}

pub struct AddonStorage {
    pool: Pool<Sqlite>,
}

pub type AddonStorageContainer = Mutex<AddonStorage>;

impl AddonStorage {
    pub async fn new(store_folder: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&store_folder).map_err(|e| e.to_string())?;
        let db_path = store_folder.join("addon-manager.db");
        std::env::set_var("DATABASE_URL", format!("sqlite://{}", db_path.display()));
        let connection_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = sqlx::sqlite::SqlitePool::connect_with(connection_options).await
            .map_err(|e| e.to_string())?;
        info!("Pool ready setup for {}", db_path.display());
        Ok(Self {
            pool
        })
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        info!("Running migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        info!("Completed migrations");
        Ok(())
    }

    pub async fn scan(&mut self, path_buf: PathBuf) -> Result<(), String> {
        info!("Scanning addons at {}", path_buf.display());
        let dir = std::fs::read_dir(path_buf).map_err(|e| e.to_string())?;
        for file in dir {
            let file = file.map_err(|e| e.to_string())?;
            let path = file.path();
            if let Some(ext) = path.extension() {
                if ext == "vpk" {
                    if let Err(e) = self._scan_file(&path, AddonFlags(0)).await {
                        error!("Failed to scan {}: {}", path.display(), e);
                    };
                }
            }
        }
        Ok(())
    }

    async fn _scan_file(&mut self, path: &PathBuf, flags: AddonFlags) -> Result<(), sqlx::Error> {
        let file = std::fs::File::open(path).unwrap();
        let file_name = path.file_name().unwrap();
        let metadata = file.metadata().unwrap();

        // TODO: check if has workshop ID

        let entry = AddonEntry {
            filename: file_name.to_string_lossy().to_string(),
            updated_at: metadata.modified().unwrap().into(),
            created_at: metadata.created().unwrap().into(),
            file_size: metadata.size() as i64,
            workshop_info: None,
            flags,
            tags: vec![],
        };

        self._add_entry(entry).await?;
        Ok(())
    }

    async fn _add_entry(&mut self, entry: AddonEntry) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO addons (filename, updated_at, created_at, file_size, flags, workshop_id) VALUES (?, ?, ?, ?, ?, ?)",
            entry.filename,
            entry.updated_at,
            entry.created_at,
            entry.file_size,
            entry.flags.0,
            None::<i64>
        ).execute(&self.pool).await?;
        info!("Added entry {} (flags={}) (ws_id={:?})", entry.filename, entry.flags.0, None::<i64>);
        Ok(())
    }
}
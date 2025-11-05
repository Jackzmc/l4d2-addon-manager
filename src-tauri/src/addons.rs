use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::SystemTime;
use bitflags::bitflags;
use chrono::DateTime;
use l4d2_addon_parser::AddonInfo;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, Pool, Sqlite};
use sqlx::types::chrono;
use sqlx::types::chrono::Utc;
use tauri::async_runtime::Mutex;
use crate::models::addon::AddonWithTagsList;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkshopItem {
    publishedfileid: String,
    title: String
}

impl WorkshopItem {
    /// Link to the steam workshop
    pub fn link(&self) -> String {
        format!("http://www.steamcommunity.com/sharedfiles/filedetails/?id={}", self.publishedfileid)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddonFlags(pub u32);
bitflags! {
    impl AddonFlags: u32 {
        /// Is addon in the 'workshop' folder
        const Workshop = 0b0001;
        /// Is addon a campaign
        const Campaign = 0b0010;
        /// Changes a survivor
        const Survivor = 0b0100;
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AddonData {
    /// Name of the file addon was found in
    pub filename: String,
    /// When addon file was last updated
    pub updated_at: chrono::DateTime<Utc>,
    /// When addon file was created
    pub created_at: chrono::DateTime<Utc>,
    /// The size in bytes of the addon file
    pub file_size: i64,

    /// The flags parsed from the addon
    pub flags: AddonFlags,
    /// Title of addon
    pub title: String,
    /// Author of addon
    pub author: Option<String>,
    /// Version of addon
    pub version: String,
    /// A short description of addon
    pub tagline: Option<String>,

    /// Extracted from either addoninfo.txt url or filename
    pub workshop_id: Option<i64>,
}



#[derive(Serialize)]
pub struct AddonEntry {
    /// Info about addon and its file
    pub addon: AddonData,
    /// If a workshop entry is linked, its contents here
    pub workshop_info: Option<WorkshopItem>,
    /// A list of user added tags for entry
    pub tags: Vec<String>,
}

pub struct AddonStorage {
    pool: Pool<Sqlite>,
}

pub type AddonStorageContainer = Mutex<AddonStorage>;

impl AddonStorage {
    pub async fn new(store_folder: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&store_folder).map_err(|e| e.to_string())?;
        let db_path = store_folder.join("addon-manager.db");
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

    pub async fn list(&self, flags: AddonFlags) -> Result<Vec<AddonEntry>, sqlx::Error> {
        Ok(sqlx::query_as::<_, AddonWithTagsList>(r#"
                select addons.*, GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.filename = addons.filename
                group by addons.filename
            "#
        )
            .bind(flags.0)
            .fetch_all(&self.pool).await?
            .into_iter().map(|entry| {
                // Skip empty strings as they have no tags
                let tags: Vec<String> = if entry.tags != "" {
                    entry.tags.split(',').map(|s| s.to_string()).collect()
                } else {
                    vec![]
                };

                AddonEntry {
                    addon: AddonData {
                        filename: entry.filename,
                        updated_at: entry.updated_at,
                        created_at: entry.created_at,
                        file_size: entry.file_size,
                        flags: AddonFlags(entry.flags),
                        title: "".to_string(),
                        author: None,
                        version: "".to_string(),
                        tagline: None,
                        workshop_id: entry.workshop_id,
                    },
                    workshop_info: None,
                    tags
                }
            })
            .collect::<Vec<AddonEntry>>())
    }

    pub async fn get_by_filename(&self, filename: &str) -> Result<Option<AddonWithTagsList>, sqlx::Error> {
        sqlx::query_as::<_, AddonWithTagsList>(r#"
                select addons.*, GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.filename = addons.filename
                where addons.filename = ?
                group by addons.filename
            "#
        )
               .bind(filename)
               .fetch_optional(&self.pool)
               .await
    }

    pub async fn get_by_pk(&self, title: &str, version: &str) -> Result<Option<AddonWithTagsList>, sqlx::Error> {
        sqlx::query_as::<_, AddonWithTagsList>(r#"
                select addons.*, GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.filename = addons.filename
                where addons.title = ? AND addons.version = ?
                group by addons.filename
            "#
        )
            .bind(title)
            .bind(version)
            .fetch_optional(&self.pool)
            .await
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
            addon: AddonData {
                filename: file_name.to_string_lossy().to_string(),
                updated_at: metadata.modified().unwrap().into(),
                created_at: metadata.created().unwrap().into(),
                file_size: metadata.size() as i64,
                flags,
                title: "".to_string(),
                author: None,
                version: "".to_string(),
                tagline: None,
                workshop_id: None,
            },
            tags: vec![],
            workshop_info: None,
        };

        self._add_entry(entry).await?;
        Ok(())
    }

    pub async fn update_entry(&mut self, filename: &str, file_meta: Metadata, addon: AddonInfo) -> Result<(), sqlx::Error> {
        let last_modified: DateTime<Utc> = file_meta.modified().unwrap().into();
        let size = file_meta.size() as i64;
        sqlx::query!(
            "UPDATE addons SET file_size = ?, updated_at = ?, title = ?, version = ? WHERE filename = ?",
            size,
            last_modified,
            addon.title,
            addon.version,
            filename
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }


    pub async fn update_entry_pk(&mut self, title: &str, version: &str, new_filename: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE addons SET filename = ? WHERE title = ? AND version = ?",
            new_filename,
            title,
            version
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn _add_entry(&mut self, entry: AddonEntry) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO addons (filename, updated_at, created_at, file_size, flags, workshop_id) VALUES (?, ?, ?, ?, ?, ?)",
            entry.addon.filename,
            entry.addon.updated_at,
            entry.addon.created_at,
            entry.addon.file_size,
            entry.addon.flags.0,
            None::<i64>
        ).execute(&self.pool).await?;
        info!("Added entry {} (flags={}) (ws_id={:?})", entry.addon.filename, entry.addon.flags.0, None::<i64>);
        Ok(())
    }
}


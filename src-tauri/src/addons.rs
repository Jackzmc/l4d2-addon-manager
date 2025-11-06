use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use bitflags::bitflags;
use chrono::DateTime;
use l4d2_addon_parser::AddonInfo;
use log::{info};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool,Sqlite};
use sqlx::types::chrono;
use sqlx::types::chrono::Utc;
use tauri::async_runtime::Mutex;
use crate::models::addon::{FullAddonWithTagsList, PartialAddon};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkshopItem {
    publishedfileid: String,
    title: String
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddonFlags(pub u32);
bitflags! {
    impl AddonFlags: u32 {
        /// Is addon in the 'workshop' folder
        const WORKSHOP = 0b0000001;
        /// Is addon a campaign
        const CAMPAIGN = 0b0000010;
        /// Changes a survivor
        const SURVIVOR = 0b0000100;
        /// Changes / adds a script
        const SCRIPT = 0b0001000;
        /// Includes a texture change
        const SKIN = 0b0010000;
        /// Weapon change
        const WEAPON = 0b0100000;
    }
}
// Needed for sqlx to load
impl From<u32> for AddonFlags {
    fn from(flags: u32) -> Self {
        AddonFlags(flags)
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AddonData {
    /// Name of the file addon was found in
    pub filename: String,
    /// When addon file was last updated
    pub updated_at: chrono::DateTime<Utc>,
    /// When addon file was created
    pub created_at: chrono::DateTime<Utc>,
    /// The size in bytes of the addon file
    pub file_size: i64,

    #[sqlx(try_from = "u32")]
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
    /// Comma separated list of chapter ids, if map
    pub chapter_ids: Option<String>,
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

pub type AddonStorageContainer = Arc<Mutex<AddonStorage>>;

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
        Ok(sqlx::query_as::<_, FullAddonWithTagsList>(r#"
                select addons.*, GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.title = addons.title AND tags.version = addons.version
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
                    addon: entry.data,
                    workshop_info: None,
                    tags
                }
            })
            .collect::<Vec<AddonEntry>>())
    }

    pub async fn get_by_filename(&self, filename: &str) -> Result<Option<PartialAddon>, sqlx::Error> {
        sqlx::query_as::<_, PartialAddon>(r#"
                select
                    addons.filename,
                    addons.updated_at, addons.created_at,
                    addons.file_size, addons.flags,
                    addons.workshop_id,
                    GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.title = addons.title AND tags.version = addons.version
                where addons.filename = ?
                group by addons.filename
            "#
        )
               .bind(filename)
               .fetch_optional(&self.pool)
               .await
    }

    pub async fn get_by_pk(&self, title: &str, version: &str) -> Result<Option<PartialAddon>, sqlx::Error> {
        sqlx::query_as::<_, PartialAddon>(r#"
                select
                    addons.filename,
                    addons.updated_at, addons.created_at,
                    addons.file_size, addons.flags,
                    addons.workshop_id,
                    GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.title = addons.title AND tags.version = addons.version
                where addons.title = ? AND addons.version = ?
                group by addons.filename
            "#
        )
            .bind(title)
            .bind(version)
            .fetch_optional(&self.pool)
            .await
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


    /// Update the entry by its primary key. Returns boolean if an entry existed and had its filename changed, false if not
    pub async fn update_entry_pk(&mut self, title: &str, version: &str, new_filename: &str) -> Result<bool, sqlx::Error> {
        let affected = sqlx::query!(
            "UPDATE addons SET filename = ? WHERE title = ? AND version = ?",
            new_filename,
            title,
            version
        )
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected > 0)
    }

    pub async fn add_entry(&mut self, addon: AddonData) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO addons
                (filename, updated_at, created_at, file_size, title, author, version, tagline, flags, workshop_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            addon.filename,
            addon.updated_at,
            addon.created_at,
            addon.file_size,
            addon.title,
            addon.author,
            addon.version,
            addon.tagline,
            addon.flags.0,
            addon.workshop_id
        ).execute(&self.pool).await?;
        info!("Added entry {} (flags={}) (ws_id={:?}) (title={})", addon.filename, addon.flags.0, addon.workshop_id, addon.title);
        Ok(())
    }
}


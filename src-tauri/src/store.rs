use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use bitflags::bitflags;
use chrono::DateTime;
use l4d2_addon_parser::AddonInfo;
use log::{info};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, QueryBuilder, Sqlite};
use sqlx::types::chrono;
use sqlx::types::chrono::Utc;
use steam_workshop_api::WorkshopItem;
use tauri::async_runtime::Mutex;
use crate::models::addon::{FullAddonWithTagsList, PartialAddon, WorkshopEntry};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddonFlags(pub u32);
bitflags! {
    impl AddonFlags: u32 {
        /// Is addon in the 'workshop' folder
        const WORKSHOP = 1;
        /// Is addon a campaign
        const CAMPAIGN = 1 << 1;
        /// Changes a survivor
        const SURVIVOR = 1 << 2;
        /// Changes / adds a script
        const SCRIPT = 1 << 3;
        /// Includes a texture change
        const SKIN = 1 << 4;
        /// Weapon change
        const WEAPON = 1 << 5;
        /// Audio, music, sound
        const SOUND = 1 << 6;
    }
}
// Needed for sqlx to load
impl From<u32> for AddonFlags {
    fn from(flags: u32) -> Self {
        AddonFlags(flags)
    }
}

#[derive(Debug, sqlx::Type, PartialEq)]
#[sqlx(transparent)]
pub struct FileHash(pub Vec<u8>);
impl Display for FileHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0))
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
    pub workshop_info: Option<WorkshopEntry>,
    /// A list of user added tags for entry
    pub tags: Vec<String>,
}

pub struct AddonStorage {
    pool: Pool<Sqlite>,
    db_path: PathBuf
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
            pool,
            db_path
        })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        info!("Running migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        info!("Completed migrations");
        Ok(())
    }

    /// Returns (# of addons, # of workshop items)
    pub async fn counts(&self) -> Result<(u32, u32), sqlx::Error> {
        let total = sqlx::query_as::<_, (u32, u32)>(
            r#"select (select count(*) from addons), (select count(*) from workshop_items where src = 'workshop')"#
        )
            .fetch_one(&self.pool).await?;
        Ok(total)
    }

    pub async fn list(&self) -> Result<Vec<AddonEntry>, sqlx::Error> {
        // TODO: include workshop_items.*
        Ok(sqlx::query_as::<_, FullAddonWithTagsList>(r#"
                select addons.*, GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.hash = addons.file_hash
                left join workshop_items wi on wi.publishedfileid = addons.workshop_id
                group by addons.file_hash
            "#
        )
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

    pub async fn list_workshop(&self) -> Result<Vec<AddonEntry>, sqlx::Error> {
        Ok(sqlx::query_as::<_, WorkshopEntry>(r#"
                select *
                from workshop_items
                where src = 'workshop'
                order by time_updated desc
            "#
        )
            .fetch_all(&self.pool).await?
            .into_iter().map(|entry| AddonEntry {
                addon: AddonData {
                    filename: format!("{}.vpk", entry.publishedfileid),
                    created_at: chrono::DateTime::from_timestamp_secs(entry.time_created).unwrap(),
                    updated_at: chrono::DateTime::from_timestamp_secs(*entry.time_updated.as_ref().unwrap()).unwrap(),
                    file_size: entry.file_size as i64,
                    flags: AddonFlags(0),
                    title: entry.title.clone(),
                    author: Some(entry.creator_id.to_string()),
                    version: "workshop".to_string(),
                    tagline: None,
                    chapter_ids: None,
                    workshop_id: Some(entry.publishedfileid as i64),
                },
                tags: entry.tags.split(',').map(|s| s.to_string()).collect(),
                workshop_info: Some(entry),
            })
            .collect::<Vec<AddonEntry>>())
    }

    pub async fn list_workshop_ids(&self) -> Result<Vec<i64>, sqlx::Error> {
        sqlx::query!(r#"
                select publishedfileid
                from workshop_items
            "#
        )
            .map(|row| row.publishedfileid)
            .fetch_all(&self.pool).await
    }

    pub async fn get_by_filename(&self, filename: &str) -> Result<Option<PartialAddon>, sqlx::Error> {
        sqlx::query_as::<_, PartialAddon>(r#"
                select
                    addons.filename,
                    addons.updated_at, addons.created_at,
                    addons.file_size, addons.flags,
                    addons.workshop_id,
                    addons.file_hash,
                    GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.hash = addons.file_hash
                where addons.filename = ?
                group by addons.filename
            "#
        )
               .bind(filename)
               .fetch_optional(&self.pool)
               .await
    }

    pub async fn update_entry(&mut self, filename: &str, hash: FileHash, file_meta: Metadata, addon: &AddonInfo, scan_id: Option<u32>) -> Result<(), sqlx::Error> {
        let last_modified: DateTime<Utc> = file_meta.modified().unwrap().into();
        let size = file_meta.size() as i64;
        sqlx::query!(
            "UPDATE addons SET file_size = ?, file_hash = ?, updated_at = ?, title = ?, version = ?, scan_id = ? WHERE filename = ?",
            size,
            hash,
            last_modified,
            addon.title,
            addon.version,
            scan_id,
            filename
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }


    /// Update the entry by its hash. Returns boolean if an entry existed and had its filename & content changed, false if not
    pub async fn update_entry_by_hash(&mut self, hash: &FileHash, new_filename: &str, title: &str, version: &str,  scan_id: Option<u32>) -> Result<bool, sqlx::Error> {
        // TODO: where count(*) is 1? if possible? to prevent multiple entries with same title/version
        let affected = sqlx::query!(
            "UPDATE addons SET filename = ?, title = ?, version = ?, scan_id = ? WHERE file_hash = ?",
            new_filename,
            title,
            version,
            scan_id,
            hash
        )
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected > 0)
    }

    /// Adds a new entry to database
    pub async fn add_entry(&self, addon: &AddonData, scan_id: Option<u32>, hash: FileHash) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO addons
                (filename, updated_at, created_at, file_size, title, author, version, tagline, flags, workshop_id, scan_id, file_hash)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            addon.workshop_id,
            scan_id,
            hash
        ).execute(&self.pool).await?;
        info!("Added entry {} (flags={}) (ws_id={:?}) (title={})", addon.filename, addon.flags.0, addon.workshop_id, addon.title);
        Ok(())
    }

    /// Attempts to add workshop items to db, overwriting existing if found
    pub async fn add_workshop_items(&self, items: Vec<WorkshopItem>, src: String, scan_id: Option<u32>) -> Result<(), sqlx::Error> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT OR REPLACE INTO workshop_items (publishedfileid, title, time_created, time_updated, file_size, description, file_url, creator_id, tags, src, scan_id) "
        );
        let num_items = items.len();
        query_builder.push_values(items, |mut b, item| {
            b.push_bind(item.publishedfileid)
                .push_bind(item.title)
                .push_bind(item.time_created as i64)
                .push_bind(item.time_updated as i64)
                .push_bind(item.file_size)
                .push_bind(item.description)
                .push_bind(item.file_url)
                .push_bind(item.creator)
                .push_bind(item.tags.iter().map(|tag| tag.tag.clone()).collect::<Vec<String>>().join(","))
                .push_bind(src.clone())
                .push_bind(scan_id.as_ref().unwrap());
        });

        let query = query_builder.build();
        query.execute(&self.pool).await?;
        info!("Added {} workshop items to database", num_items);
        Ok(())
    }

    /// Sets filenames to null for any entry that does not match scan_id
    /// To be called at end of scan
    pub async fn scan_mark_missing(&self, id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE addons SET filename = NULL WHERE scan_id != ?", id)
            .execute(&self.pool).await?;
        sqlx::query!("UPDATE workshop_items SET src = '' WHERE scan_id != ?", id)
            .execute(&self.pool).await?;
        Ok(())
    }

    /// Wipes all data from database
    pub async fn danger_delete(&self) -> Result<(), std::io::Error> {
        self.pool.close().await;
        fs::remove_file(&self.db_path)
    }

}


use crate::models::addon::{StandardAddonWithTags, WorkshopEntry};
use bitflags::bitflags;
use chrono::DateTime;
use l4d2_addon_parser::AddonInfo;
use l4d2_addon_parser::addon_list::AddonList;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use sqlx::types::chrono::Utc;
use sqlx::{AssertSqlSafe, FromRow, Pool, QueryBuilder, Sqlite};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use hex::FromHexError;
use steam_workshop_api::WorkshopItem;
use tauri::async_runtime::Mutex;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
/// for standard addons
pub struct AddonFlags(pub u32);
bitflags! {
    impl AddonFlags: u32 {
        /// Is entry from the 'workshop' folder
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

impl FileHash {
    pub fn from_str(s: &str) -> Result<Self, FromHexError> {
        hex::decode(s).map(FileHash)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
/// Information about the addon. This is used by both standard entries and workshop entries
pub struct AddonData {
    /// Name of the file addon was found in
    pub filename: String,
    /// When addon file was last updated
    pub updated_at: DateTime<Utc>,
    /// When addon file was created
    pub created_at: DateTime<Utc>,
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
    /// ID of addon, either workshop id or file hash
    pub id: String,
    /// Info about addon and its file
    pub info: AddonData,
    /// If a workshop entry is linked, its contents here
    pub workshop: Option<WorkshopEntry>,
    /// A list of user added tags for entry
    pub tags: Vec<String>,
    /// Is addon enabled? Can be None if file missing
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct SelectedSort {
    field: String,
    descending: bool,
}
impl SelectedSort {
    pub fn new(field: &str, descending: bool) -> Self {
        Self { field: field.to_string(), descending }
    }

    pub fn get_sql(&self) -> String {
        format!("{} {}", self.field, if self.descending { "DESC" } else { "ASC" })
    }
}

pub struct AddonStorage {
    pool: Pool<Sqlite>,
    db_path: PathBuf,
}

pub type AddonStorageContainer = Arc<Mutex<AddonStorage>>;

impl AddonStorage {
    pub async fn new(store_folder: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&store_folder).map_err(|e| e.to_string())?;
        let db_path = store_folder.join("addon-manager.db");
        let connection_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = sqlx::sqlite::SqlitePool::connect_with(connection_options)
            .await
            .map_err(|e| e.to_string())?;
        info!("Pool ready setup for {}", db_path.display());
        Ok(Self { pool, db_path })
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
        // flags & 1 marks AddonFlags::WORKSHOP
        let total = sqlx::query_as::<_, (u32, u32)>(
            r#"select (select count(*) from addons), (select count(*) from workshop_items where flags & 1)"#
        )
            .fetch_one(&self.pool).await?;
        Ok(total)
    }

    pub async fn list(
        &self,
        addon_list: Option<AddonList>,
        sort: Option<SelectedSort>
    ) -> Result<Vec<AddonEntry>, sqlx::Error> {
        // TODO: include workshop_items.*
        let sort = sort.unwrap_or(SelectedSort { field: "title".to_string(), descending: true });
        debug!("Sorting by {}", sort.get_sql());
        Ok(sqlx::query_as::<_, StandardAddonWithTags>(
            AssertSqlSafe(format!("
                select addons.*, GROUP_CONCAT(tags.tag) tags
                from addons
                left join addon_tags tags on tags.hash = addons.file_hash
                left join workshop_items wi on wi.publishedfileid = addons.workshop_id
                group by addons.file_hash
                order by {}
            ", sort.get_sql())),
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|entry| {
            // Skip empty strings as they have no tags
            let tags: Vec<String> = if entry.tags != "" {
                entry.tags.split(',').map(|s| s.to_string()).collect()
            } else {
                vec![]
            };
            AddonEntry {
                id: entry.file_hash.to_string(),
                enabled: addon_list
                    .as_ref()
                    .map(|list| list.is_enabled(&entry.data.filename)),
                info: entry.data,
                workshop: None,
                tags,
            }
        })
        .collect::<Vec<AddonEntry>>())
    }

    pub async fn list_workshop(
        &self,
        addon_list: Option<AddonList>,
        sort: Option<SelectedSort>
    ) -> Result<Vec<AddonEntry>, sqlx::Error> {
        // flags & 1 marks AddonFlags::WORKSHOP
        let sort = sort.unwrap_or(SelectedSort { field: "time_updated".to_string(), descending: true });
        debug!("Sorting by {}", sort.get_sql());
        Ok(sqlx::query_as::<_, WorkshopEntry>(
            AssertSqlSafe(format!(r#"
                select *
                from workshop_items
                where flags & 1
                order by {}
            "#, sort.get_sql()))
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|entry| AddonEntry {
            id: entry.publishedfileid.to_string(),
            enabled: addon_list
                .as_ref()
                .map(|list| list.is_enabled(&format!("workshop\\{}.vpk", entry.publishedfileid))),
            info: AddonData {
                filename: format!("{}.vpk", entry.publishedfileid),
                created_at: chrono::DateTime::from_timestamp_secs(entry.time_created).unwrap(),
                updated_at: chrono::DateTime::from_timestamp_secs(
                    *entry.time_updated.as_ref().unwrap(),
                )
                .unwrap(),
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
            workshop: Some(entry),
        })
        .collect::<Vec<AddonEntry>>())
    }

    pub async fn list_workshop_ids(&self) -> Result<Vec<i64>, sqlx::Error> {
        sqlx::query!(
            r#"
                select publishedfileid
                from workshop_items
            "#
        )
        .map(|row| row.publishedfileid)
        .fetch_all(&self.pool)
        .await
    }

    /// Update the entry by its hash. Returns boolean if an entry existed and had its filename & content changed, false if not
    pub async fn update_entry_by_hash(
        &mut self,
        hash: &FileHash,
        new_filename: &str,
        info: &AddonInfo,
        scan_id: Option<u32>,
    ) -> Result<bool, sqlx::Error> {
        let flags: AddonFlags = (&info.content).into();
        let affected = sqlx::query!(
            "UPDATE addons SET filename = ?, title = ?, version = ?, flags = ?, scan_id = ? WHERE file_hash = ?",
            new_filename,
            info.title,
            info.version,
            flags.0,
            scan_id,
            hash,
        )
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected > 0)
    }

    pub async fn update_entry_by_filename (
        &mut self,
        new_hash: &FileHash,
        filename: &str,
        info: &AddonInfo,
        scan_id: Option<u32>,
    ) -> Result<bool, sqlx::Error> {
        let flags: AddonFlags = (&info.content).into();
        let affected = sqlx::query!(
            "UPDATE addons SET file_hash = ?, title = ?, version = ?, flags = ?, scan_id = ? WHERE filename = ?",
            new_hash,
            info.title,
            info.version,
            flags.0,
            scan_id,
            filename,
        )
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected > 0)
    }

    /// Adds a new entry to database
    pub async fn add_entry(
        &self,
        addon: &AddonData,
        scan_id: Option<u32>,
        hash: FileHash,
    ) -> Result<(), sqlx::Error> {
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
        info!(
            "Added entry {} (flags={}) (ws_id={:?}) (title={})",
            addon.filename, addon.flags.0, addon.workshop_id, addon.title
        );
        Ok(())
    }

    /// Attempts to add workshop items to db, overwriting existing if found
    pub async fn add_workshop_items(&self, items: Vec<WorkshopItem>) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT OR REPLACE INTO workshop_items (publishedfileid, title, time_created, time_updated, file_size, description, file_url, creator_id, tags) ",
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
                .push_bind(
                    item.tags
                        .iter()
                        .map(|tag| tag.tag.clone())
                        .collect::<Vec<String>>()
                        .join(","),
                );
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
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_workshop_ids(&self, ids: Vec<i64>) -> Result<(), sqlx::Error> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        // Drop AddonFlags::WORKSHOP for all items
        sqlx::query!("UPDATE workshop_items SET flags=flags&~1 WHERE flags & 1")
            .execute(&mut *tx)
            .await?;
        // Set all given ids to include AddonFlags::WORKSHOP
        let params = format!("?{}", ", ?".repeat(ids.len() - 1));
        let mut query = sqlx::query(AssertSqlSafe(format!(
            "UPDATE workshop_items SET flags=flags|1 WHERE publishedfileid IN ({})",
            params
        )));
        for id in ids {
            query = query.bind(id);
        }
        query.execute(&mut *tx).await?;
        tx.commit().await
    }

    pub async fn delete_filenames(&self, filenames: Vec<String>) -> Result<(), sqlx::Error> {
        let params = format!("?{}", ", ?".repeat(filenames.len() - 1));
        // dynamically add ?, ?, ?... to number of filenames
        let mut query = sqlx::query(AssertSqlSafe(format!(
            "DELETE FROM addons WHERE filename IN ({})",
            params
        )));
        for filename in filenames {
            query = query.bind(filename);
        }
        query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn add_tag(&self, hash: FileHash, tag: String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO addon_tags (hash, tag) VALUES (?, ?)",
            hash, tag
        )
            .execute(&self.pool).await
            .map(|_| ())

    }

    pub async fn del_tag(&self, hash: FileHash, tag: String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM addon_tags WHERE hash = ? AND tag = ?",
            hash, tag
        )
            .execute(&self.pool).await
            .map(|_| ())
    }

    /// Wipes all data from database
    pub async fn danger_drop_database(&self) -> Result<(), std::io::Error> {
        self.pool.close().await;
        fs::remove_file(&self.db_path)
    }
}

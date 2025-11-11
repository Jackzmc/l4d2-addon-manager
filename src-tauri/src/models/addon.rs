use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::store::{AddonData, FileHash};

#[derive(Debug, FromRow)]
pub struct PartialAddon {
    pub filename: String,
    pub updated_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub file_size: i64,
    /// SHA512 hash of file
    pub file_hash: FileHash,
    pub flags: u32,
    pub workshop_id: Option<i64>,
}

#[derive(Debug, FromRow)]
pub struct StandardAddonWithTags {
    #[sqlx(flatten)]
    pub data: AddonData,

    pub file_hash: FileHash,

    /// Comma separated list of tags
    pub tags: String
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WorkshopEntry {
    pub publishedfileid: u32,
    pub title: String,
    pub time_created: i64,
    pub time_updated: Option<i64>,
    pub file_size: u32,
    pub description: String,
    pub file_url: String,
    pub creator_id: String,
    pub tags: String,
}
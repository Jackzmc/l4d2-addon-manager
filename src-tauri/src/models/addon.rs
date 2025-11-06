use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::addons::{AddonData};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PartialAddon {
    pub filename: String,
    pub updated_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub file_size: i64,
    pub flags: u32,
    pub workshop_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FullAddonWithTagsList {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub data: AddonData,

    /// Comma separated list of tags
    pub tags: String
}

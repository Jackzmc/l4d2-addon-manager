use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::addons::AddonFlags;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AddonWithTagsList {
    pub filename: String,
    pub updated_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub file_size: i64,
    pub flags: u32,
    pub workshop_id: Option<i64>,

    /// Comma separated list of tags
    pub tags: String
}

use std::hash::Hash;
use l4d2_addon_parser::L4D2Addon;
use l4d2_addon_parser::AddonInfo;
use crate::scan::helpers::find_workshop_id;
use crate::scan::helpers::get_addon_flags;
use chrono::Utc;
use chrono::DateTime;
use crate::store::{AddonData, FileHash};
use std::sync::atomic::Ordering;
use crate::scan::ScanCounter;
use std::sync::Arc;
use crate::store::AddonStorageContainer;
use std::path::PathBuf;
use steam_workshop_api::WorkshopItem;
use log::{error, trace};
use serde::Serialize;
use log::debug;
use std::os::unix::fs::MetadataExt;
use sqlx::__rt::spawn_blocking;

pub(super) enum WorkerOutput {
    /// Worker has new workshop id to enqueue
    WorkshopId(i64),
    /// Worker has fetched workshop items
    WorkshopItems(Vec<WorkshopItem>),
    /// Worker has nothing useful
    None
}

pub(super) async fn scan_file_wrapper(path: PathBuf, addons: AddonStorageContainer, counter: Arc<ScanCounter>, scan_id: u32) -> WorkerOutput {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    counter.total.fetch_add(1, Ordering::Relaxed);
    match scan_file(path, addons, scan_id).await {
        Ok((result, data)) => {
            match result {
                ScanResult::Added => {
                    counter.added.fetch_add(1, Ordering::Relaxed);
                    if let Some(ws_id) = data.expect("added has entries").workshop_id {
                        return WorkerOutput::WorkshopId(ws_id)
                    }
                },
                _ => {}
            };
        },
        Err(err) => {
            error!("SCAN ERROR FOR \"{}\": {}", filename, err);
            counter.errors.fetch_add(1, Ordering::Relaxed);
        }
    }
    WorkerOutput::None
}

#[derive(Debug)]
pub enum ScanResult {
    /// Existing addon found by filename, no changes
    NoChanges,
    /// Existing addon has changed, its info has been updated
    Updated,
    /// Existing addon found by hash, updated filename
    Renamed,
    /// No existing addon found, new addon added
    Added,

}

pub enum ScanError {
    FileError(std::io::Error),
    ParseError(l4d2_addon_parser::Error),
    DBError(sqlx::Error),
    UpdateError(sqlx::Error),
    UpdateRenameError(sqlx::Error),
    NewEntryError(sqlx::Error),
}

async fn scan_file(path: PathBuf, addons: AddonStorageContainer, scan_id: u32) -> Result<(ScanResult, Option<AddonData>), ScanError> {
    // TODO: use hash over title,version and whatever
    let meta = path.metadata().map_err(|e| ScanError::FileError(e))?;
    let filename = path.file_name().unwrap().to_str().unwrap();
    let addon_entry = {
        let addons = addons.lock().await;
        addons.get_by_filename(filename).await
            .map_err(|e| ScanError::DBError(e))?
    };
    let (info, chapter_ids, hash) = {
        let path = path.clone();
        spawn_blocking(move || parse_addon(&path)).await
            .map_err(|e| ScanError::ParseError(e))?
    };
    trace!("scan_file \"{}\" hash \"{}\" thread {:?}", filename, hash, std::thread::current().id());

    if let Some(entry) = addon_entry {
        let last_modified = meta.modified().map_err(|e| ScanError::FileError(e))?;
        // Check if file has been modified since scanned
        if <DateTime<Utc>>::from(last_modified) > entry.updated_at || hash != entry.file_hash {
            let mut addons = addons.lock().await;
            debug!("found changed file: {}, updating", filename);
            addons.update_entry(filename, hash, meta, &info, Some(scan_id)).await
                .map_err(|e| ScanError::UpdateError(e))?;
            return Ok((ScanResult::Updated, None))
        }
        return Ok((ScanResult::NoChanges, None))
    }


    // If info has title and version, try to find previous entry and update its filename
    if let Some(title) = &info.title && let Some(version) = &info.version {
        // If we found a previous entry, we are done.
        // Next time a scan is performed any changes will be reflected by the last modified check
        debug!("possible rename. checking for a possible entry. hash={} for file=\"{}\"", hash, filename);
        let mut addons = addons.lock().await;
        if addons.update_entry_by_hash(&hash, filename, version, title, Some(scan_id)).await
            .map_err(|e| ScanError::UpdateRenameError(e))?
        {
            debug!("found renamed file: {}, hash: {}", filename, hash);
            return Ok((ScanResult::Renamed, None))
        }
    }

    let flags = get_addon_flags(&info);
    let ws_id = find_workshop_id(&path, &info);

    debug!("found new addon: {}", filename);

    // Treat file as new now
    let data = AddonData {
        filename: filename.to_string(),
        updated_at: meta.modified().map_err(|e| ScanError::FileError(e))?.into(),
        created_at: meta.created().map_err(|e| ScanError::FileError(e))?.into(),
        file_size: meta.size() as i64,
        flags,
        title: info.title.unwrap(), // TODO: if no info/info.title, use filename?
        author: info.author,
        version: info.version.unwrap(),
        tagline: info.tagline,
        chapter_ids: chapter_ids.map(|c| c.join(",")),
        workshop_id: ws_id
    };

    // Add to DB
    let addons = addons.lock().await;
    addons.add_entry(&data, Some(scan_id), hash).await
        .map_err(|e| ScanError::NewEntryError(e))?;

    Ok((ScanResult::Added, Some(data)))
}

/// returns info, missions, and hash (bytes)
pub fn parse_addon(path: &PathBuf) -> Result<(AddonInfo, Option<Vec<String>>, FileHash), l4d2_addon_parser::Error> {
    let mut addon = L4D2Addon::from_path(&path)?;
    let info = addon.info()?
        .ok_or(l4d2_addon_parser::Error::VPKError("Bad addon: No addoninfo.txt found in addon".to_string()))?;

    let mut chapter_ids: Option<Vec<String>> = None;
    if let Some(mission) = addon.missions()? {
        if let Some(coop) = mission.modes.get("coop") {
            chapter_ids = Some(coop.iter().map(|entry| entry.1.map.clone()).collect());
        }
    }

    let hash = addon.hash_256()?;

    Ok((info, chapter_ids, FileHash(hash)))
}
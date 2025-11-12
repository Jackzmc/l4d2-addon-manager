use std::collections::VecDeque;
use l4d2_addon_parser::L4D2Addon;
use l4d2_addon_parser::AddonInfo;
use crate::scan::helpers::find_workshop_id;
use crate::modules::store::{AddonData, AddonFlags, FileHash};
use std::sync::Arc;
use crate::modules::store::AddonStorageContainer;
use std::path::PathBuf;
use std::time::Instant;
use log::{error, trace, warn};
use log::debug;
use steam_workshop_api::{SteamWorkshop, WorkshopItem};
use crate::util::get_file_size;

#[derive(Debug)]
pub enum ProcessResult {
    /// Existing addon found by hash, its info has been updated
    UpdatedByHash,
    /// No existing addon found, new addon added
    Added,
}

pub enum ProcessError {
    FileError(std::io::Error),
    UpdateExistingError(sqlx::Error),
    NewEntryError(sqlx::Error),
}

pub struct AddonFileData {
    path: PathBuf,
    filename: String,
    info: AddonInfo,
    chapter_ids: Option<Vec<String>>,
    hash: FileHash,
}
pub enum WorkerTask {
    /// Worker should scan a file
    ScanFile(PathBuf),
}
// pub(super) enum WorkerOutput {
//     /// Worker has new workshop id to enqueue
//     ScanResult(AddonFileData),
//     /// Worker has nothing useful
//     None
// }
pub fn scan_worker_thread(i: u8, tx: tokio::sync::mpsc::Sender<Result<AddonFileData, String>>, queue: Arc<tokio::sync::Mutex<VecDeque<WorkerTask>>>) -> std::io::Result<()> {
    loop {
        let task = {
            let mut queue = queue.blocking_lock();
            queue.pop_front()
        };
        match task {
            Some(WorkerTask::ScanFile(path)) => {
                let time = Instant::now();
                match scan_file(path) {
                    Ok(res) => {
                        trace!("[worker{i}] scan_file \"{}\" hash \"{}\" took {}ms", &res.filename, &res.hash, time.elapsed().as_millis());
                        tx.blocking_send(Ok(res)).expect("failed to send result");
                    },
                    Err(e) => {
                        warn!("[worker{i}] scan_file failed: {}", e);
                    }
                }
            },
            None => break
        }
    }
    trace!("[worker{i}] done. exiting");
    Ok(())
}
/// processes list of workshop ids in batches of 100 and returns full list of workshop items
pub fn scan_workshop_thread(mut ids: Vec<i64>) -> Vec<WorkshopItem> {
    let ws = SteamWorkshop::new();
    let mut results: Vec<WorkshopItem> = Vec::new();
    // Steam API only supports upto 100 at a time, so process in batches of 100
    while !ids.is_empty() {
        let items_to_drain = 100.min(ids.len()); // drain panics if over len, get smallest
        trace!("fetching slice of {items_to_drain} ids");
        let slice: Vec<String> = ids.drain(0..items_to_drain).map(|item| item.to_string()).collect();

        match ws.get_published_file_details(&slice) {
            Ok(items) => {
                if !items.is_empty() {
                    results.extend(items);
                }
            },
            Err(err) => {
                error!("failed to get workshop ids: {}", err);
            }
        }
    }
    results
}
pub fn scan_file(path: PathBuf) -> Result<AddonFileData, String> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let (info, chapter_ids, hash) = parse_addon(&path)
        .map_err(|e| format!("failed to parse addon \"{}\": {}", filename, e))?;
    Ok(AddonFileData { path, filename: filename.to_string(), info, chapter_ids, hash })
}

pub async fn async_process_file(file: AddonFileData, addons: AddonStorageContainer, scan_id: u32) -> Result<(ProcessResult, Option<i64>), ProcessError> {
    let meta = file.path.metadata().map_err(|e| ProcessError::FileError(e))?;
    trace!("process_file \"{}\"", &file.filename);

    let mut addons = addons.lock().await;
    if addons.update_entry_by_hash(&file.hash, &file.filename, &file.info, Some(scan_id)).await
        .map_err(|e| ProcessError::UpdateExistingError(e))?
    {
        debug!("found existing file: \"{}\" by hash \"{}\"", file.filename, file.hash);
        return Ok((ProcessResult::UpdatedByHash, None))
    }

    let ws_id = find_workshop_id(&file.filename, &file.info);

    // Treat file as new now
    let flags: AddonFlags = (&file.info.content).into();
    let data = AddonData {
        filename: file.filename.to_string(),
        updated_at: meta.modified().map_err(|e| ProcessError::FileError(e))?.into(),
        created_at: meta.created().map_err(|e| ProcessError::FileError(e))?.into(),
        file_size: get_file_size(&meta),
        flags: flags,
        title: file.info.title.unwrap(), // TODO: if no info/info.title, use filename?
        author: file.info.author,
        version: file.info.version.unwrap(),
        tagline: file.info.tagline,
        chapter_ids: file.chapter_ids.map(|c| c.join(",")),
        workshop_id: ws_id
    };

    // Add to DB
    addons.add_entry(&data, Some(scan_id), file.hash).await
        .map_err(|e| ProcessError::NewEntryError(e))?;

    debug!("found new addon: \"{}\"", file.filename);

    Ok((ProcessResult::Added, data.workshop_id))
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
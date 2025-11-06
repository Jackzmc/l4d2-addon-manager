use std::collections::VecDeque;
use std::fmt::Display;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::JoinHandle;
use chrono::{DateTime, Utc};
use l4d2_addon_parser::{AddonInfo, L4D2Addon, MissionInfo};
use log::{debug, error, info};
use regex::Regex;
use serde::Serialize;
use tauri::async_runtime::{channel, Receiver, Sender};
use tauri::{AppHandle, Emitter, State, Window};
use crate::addons::{AddonData, AddonEntry, AddonFlags, AddonStorageContainer};
use crate::scan::ScanError::{DBError, FileError, NewEntryError, ParseError, UpdateError, UpdateRenameError};

pub struct AddonScanner {
    thread: Option<JoinHandle<()>>,
    queue: Option<Arc<Mutex<VecDeque<PathBuf>>>>,

    addons: AddonStorageContainer,
    app: AppHandle
}

pub type ScannerContainer = Mutex<AddonScanner>;

impl AddonScanner {
    pub fn new(addons: AddonStorageContainer, app: AppHandle) -> Self {
        Self {
            thread: None,
            queue: None,
            addons,
            app
        }
    }

    /// Starts an async scan. New items will appear in database
    pub fn start_scan(&mut self, path: PathBuf) -> bool {
        if self.thread.is_none() {
            let mut queue = Arc::new(Mutex::new(VecDeque::new()));
            self.queue = Some(queue.clone());
            info!("Starting new scan, performing initial directory scan");

            self.app.emit("scan_state", ScanState::Started).ok();

            self._scan_dir(&path).expect("failed to scan dir");

            info!("Done. Starting addon scan thread");
            let counter = Arc::new(ScanCounter::default());
            let thread = {
                let queue = queue;
                let addons = self.addons.clone();
                let counter = counter.clone();
                let app = self.app.clone();
                std::thread::spawn(move || scan_thread(queue, addons, counter, app))
            };
            self.thread = Some(thread);
        }
        true
    }
    pub fn abort_scan(&mut self) {
        let queue = self.queue.as_ref().unwrap();
        let mut queue = queue.lock().unwrap();
        queue.clear();
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn is_running(&self) -> bool {
        self.thread.is_some()
    }

    /// Performs a scan of directory
    fn _scan_dir(&mut self, path: &PathBuf) -> Result<(), String> {
        info!("Scanning addons at {}", path.display());
        let dir = std::fs::read_dir(path)
            .map_err(|e| e.to_string())?;
        let mut queue = self.queue.as_ref().expect("queue not initialized")
            .lock().unwrap();
        for file in dir {
            let file = file.map_err(|e| e.to_string())?;
            let path = file.path();
            if let Some(ext) = path.extension() {
                if ext == "vpk" {
                    queue.push_front(path);
                }
            }
        }
        Ok(())
    }
}
#[derive(Default, Clone)]
struct ScanCounter {
    total: Arc<AtomicU32>,
    added: Arc<AtomicU32>,
    errors: Arc<AtomicU32>
}
type ScanCounterContainer = Mutex<ScanCounter>;

fn scan_thread(queue: Arc<Mutex<VecDeque<PathBuf>>>, addons: AddonStorageContainer, counter: Arc<ScanCounter>, app: AppHandle) {
    loop {
        let item = {
            let mut queue = queue.lock().unwrap();
            if queue.len() == 0 { break; }
            queue.pop_back().unwrap()
        };

        let addons = addons.clone();
        let counter = counter.clone();
        let app = app.clone();
        tauri::async_runtime::block_on(async move {
            counter.total.fetch_add(1, Ordering::Relaxed);
            let filename = item.file_name().unwrap().to_string_lossy().to_string();
            match scan_file(&item, addons).await {
                Ok(result) => {
                    match result {
                        ScanResult::Added => { counter.added.fetch_add(1, Ordering::Relaxed); },
                        _ => {}
                    };
                    app.emit("scan_result", ScanResultPayload {
                        result,
                        filename
                    }).ok();
                },
                Err(e) => {
                    error!("SCAN ERROR FOR \"{}\": {}", filename, e);
                    counter.errors.fetch_add(1, Ordering::Relaxed);
                },
                _ => {}
            }
        });
    }
    app.emit("scan_state", ScanState::Complete).ok();
    info!("ADDON SCAN COMPLETE. {} addons scanned, {} added, {} failed", counter.total.load(Ordering::SeqCst), counter.added.load(Ordering::SeqCst), counter.errors.load(Ordering::SeqCst));
    // TODO: send notification to UI
}
#[derive(Serialize, Clone)]
#[serde(rename = "snake_case")]
pub enum ScanState {
    Started,
    Failed,
    Complete
}

pub enum ScanError {
    FileError(std::io::Error),
    ParseError(l4d2_addon_parser::Error),
    DBError(sqlx::Error),
    UpdateError(sqlx::Error),
    UpdateRenameError(sqlx::Error),
    NewEntryError(sqlx::Error),
}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::FileError(e) => write!(f, "IO Error: {}", e),
            ScanError::ParseError(e) => write!(f, "Parse Error: {}", e),
            ScanError::UpdateError(e) => write!(f, "Error updating item: {}", e),
            ScanError::UpdateRenameError(e) => write!(f, "Error updating renamed item: {}", e),
            ScanError::NewEntryError(e) => write!(f, "Error creating new entry: {}", e),
            ScanError::DBError(e) => write!(f, "DB Error: {}", e),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ScanResultPayload {
    result: ScanResult,
    filename: String
}

#[derive(Serialize, Clone)]
#[serde(rename = "snake_case")]
pub enum ScanResult {
    Updated,
    Renamed,
    Added,
    NoAction,
}
pub async fn scan_file(path: &PathBuf, addons: AddonStorageContainer) -> Result<ScanResult, ScanError> {
    let meta = path.metadata().map_err(|e| FileError(e))?;
    let filename = path.file_name().unwrap().to_str().unwrap();
    let addon_entry = {
        let addons = addons.lock().await;
        addons.get_by_filename(filename).await
            .map_err(|e| DBError(e))?
    };

    let (info, chapter_ids) = parse_addon(&path).await
        .map_err(|e| ParseError(e))?;

    let mut should_update = false;
    if let Some(entry) = addon_entry {
        let last_modified = meta.modified().map_err(|e| FileError(e))?;
        // Check if file has been modified since scanned
        should_update = <DateTime<Utc>>::from(last_modified) > entry.updated_at;
        if <DateTime<Utc>>::from(last_modified) > entry.updated_at {
            let mut addons = addons.lock().await;
            debug!("file has changed, updating entry {:?}", path);
            addons.update_entry(filename, meta, info).await
                .map_err(|e| UpdateError(e))?;
            return Ok(ScanResult::Updated)
        }
        return Ok(ScanResult::NoAction)
    }

    let mut addons = addons.lock().await;
    // If info has title and version, try to find previous entry and update its filename
    if let Some(title) = &info.title && let Some(version) = &info.version {
        // If we found a previous entry, we are done.
        // Next time a scan is performed any changes will be reflected by the last modified check
        if addons.update_entry_pk(filename, version, title).await
            .map_err(|e| UpdateRenameError(e))?
        {
            return Ok(ScanResult::Renamed)
        }
    }

    let flags = get_addon_flags(&info);
    let ws_id = find_workshop_id(&path, &info);

    // Treat file as new now
    let data = AddonData {
            filename: filename.to_string(),
            updated_at: meta.modified().map_err(|e| FileError(e))?.into(),
            created_at: meta.created().map_err(|e| FileError(e))?.into(),
            file_size: meta.size() as i64,
            flags,
            title: info.title.unwrap(),
            author: info.author,
            version: info.version.unwrap(),
            tagline: info.tagline,
            chapter_ids: chapter_ids.map(|c| c.join(",")),
            workshop_id: ws_id
    };
    // TODO: add missions
    // Add to DB
    addons.add_entry(data).await
        .map_err(|e| NewEntryError(e))?;

    Ok(ScanResult::Added)
}

pub async fn parse_addon(path: &PathBuf) -> Result<(AddonInfo, Option<Vec<String>>), l4d2_addon_parser::Error> {
    let mut addon = L4D2Addon::from_path(&path)?;
    let info = addon.info()?
        .ok_or(l4d2_addon_parser::Error::VPKError("Bad addon: No addoninfo.txt found in addon".to_string()))?;

    let mut chapter_ids: Option<Vec<String>> = None;
    if let Some(mission) = addon.missions()? {
        if let Some(coop) = mission.modes.get("coop") {
            chapter_ids = Some(coop.iter().map(|entry| entry.1.map.clone()).collect());
        }
    }
    Ok((info, chapter_ids))
}

// Can guarantee id is 4 digits at minimum.
// IDs are sequential, L4D2 Workshop came out after the 10000th addon was released
static WORKSHOP_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https://steamcommunity.com/sharedfiles/filedetails/\?id=(\d+)").unwrap());
static WORKSHOP_FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{4,}").unwrap());


/// Attempts to extract workshop ID from addon url or filename
fn find_workshop_id(path: &PathBuf, addon: &AddonInfo) -> Option<i64> {
    // Try URL first, as we can guarantee from there
    if let Some(url) = &addon.addon_url {
        if let Some(capture) = WORKSHOP_URL_REGEX.captures(url) {
            let id = capture.get(1).unwrap().as_str();
            debug!("Found workshop ID \"{}\" (addon url)", id);
            return Some(id.parse::<i64>().unwrap());
        }
    }

    // Try to get it from filename
    let filename = path.file_name().unwrap().to_str().unwrap();
    if let Some(cap) = WORKSHOP_FILE_REGEX.find(filename) {
        let id = cap.as_str().parse::<i64>().unwrap();
        debug!("Found workshop ID \"{}\" (file)", id);
        return Some(id);
    }
    None
}

fn get_addon_flags(info: &AddonInfo) -> AddonFlags {
    let mut flags = AddonFlags::empty();
    if info.content.is_map {
        flags |= AddonFlags::CAMPAIGN;
    }
    if info.content.is_survivor {
        flags |= AddonFlags::SURVIVOR;
    }
    if info.content.is_script {
        flags |= AddonFlags::SCRIPT;
    }
    if info.content.is_weapon {
        flags |= AddonFlags::WEAPON;
    }
    flags
}
use std::collections::VecDeque;
use std::fmt::Display;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::JoinHandle;
use chrono::{DateTime, Utc};
use l4d2_addon_parser::{AddonInfo, L4D2Addon, MissionInfo};
use log::{debug, error, info};
use tauri::async_runtime::{channel, Receiver, Sender};
use tauri::State;
use crate::addons::{AddonData, AddonEntry, AddonFlags, AddonStorageContainer};
use crate::scan::ScanError::{DBError, FileError, NewEntryError, ParseError, UpdateError, UpdateRenameError};

pub struct AddonScanner {
    thread: Option<JoinHandle<()>>,
    queue: Option<Arc<Mutex<VecDeque<PathBuf>>>>,

    addons: AddonStorageContainer,
}

pub type ScannerContainer = Mutex<AddonScanner>;

impl AddonScanner {
    pub fn new(addons: AddonStorageContainer) -> Self {
        Self {
            thread: None,
            queue: None,
            addons,
        }
    }

    /// Starts an async scan. New items will appear in database
    pub fn start_scan(&mut self, path: PathBuf) -> bool {
        if self.thread.is_none() {
            let mut queue = Arc::new(Mutex::new(VecDeque::new()));
            self.queue = Some(queue.clone());
            info!("Starting new scan, performing initial directory scan");


            self._scan_dir(&path).expect("failed to scan dir");

            info!("Done. Starting addon scan thread");
            let counter = Arc::new(ScanCounter::default());
            let thread = {
                let queue = queue;
                let addons = self.addons.clone();
                let counter = counter.clone();
                std::thread::spawn(move || scan_thread(queue, addons, counter))
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

fn scan_thread(queue: Arc<Mutex<VecDeque<PathBuf>>>, addons: AddonStorageContainer, counter: Arc<ScanCounter>) {
    loop {
        let item = {
            let mut queue = queue.lock().unwrap();
            if queue.len() == 0 { break; }
            queue.pop_back().unwrap()
        };

        let addons = addons.clone();
        let counter = counter.clone();
        tauri::async_runtime::block_on(async move {
            counter.total.fetch_add(1, Ordering::Relaxed);
            match scan_file(&item, addons).await {
                Ok(ScanResult::Added) => {
                    counter.added.fetch_add(1, Ordering::Relaxed);
                },
                Err(e) => {
                    let filename = item.file_name().unwrap().to_string_lossy();
                    error!("SCAN ERROR FOR \"{}\": {}", filename, e);
                    counter.errors.fetch_add(1, Ordering::Relaxed);
                },
                _ => {}
            }
        });
    }
    // FIXME: why is it 0.
    info!("ADDON SCAN COMPLETE. {} addons scanned, {} added, {} failed", counter.total.load(Ordering::SeqCst), counter.added.load(Ordering::SeqCst), counter.errors.load(Ordering::SeqCst));
    // TODO: send notification to UI
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

pub enum ScanResult {
    Updated,
    Renamed,
    Added,
    NoAction,
    Error(ScanError)
}
pub async fn scan_file(path: &PathBuf, addons: AddonStorageContainer) -> Result<ScanResult, ScanError> {
    let meta = path.metadata().map_err(|e| FileError(e))?;
    let filename = path.file_name().unwrap().to_str().unwrap();
    let addon_entry = {
        let addons = addons.lock().await;
        addons.get_by_filename(filename).await
            .map_err(|e| DBError(e))?
    };

    let (info, missions) = parse_addon(&path).await
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
            tagline: None, //info.tagline,
            workshop_id: None
    };
    // Add to DB
    addons.add_entry(data).await
        .map_err(|e| NewEntryError(e))?;

    Ok(ScanResult::Added)
}

pub async fn parse_addon(path: &PathBuf) -> Result<(AddonInfo, Option<MissionInfo>), l4d2_addon_parser::Error> {
    let mut addon = L4D2Addon::from_path(&path)?;
    let info = addon.info()?
        .ok_or(l4d2_addon_parser::Error::VPKError("Bad addon: No addoninfo.txt found in addon".to_string()))?;
    let map = addon.missions()?;
    Ok((info, map))
}

fn get_addon_flags(info: &AddonInfo) -> AddonFlags {
    let mut flags = AddonFlags::empty();
    if info.is_map {
        flags |= AddonFlags::CAMPAIGN;
    }
    if info.is_survivor {
        flags |= AddonFlags::SURVIVOR;
    }
    if info.is_script {
        flags |= AddonFlags::SCRIPT;
    }
    if info.is_weapon {
        flags |= AddonFlags::WEAPON;
    }
    flags
}
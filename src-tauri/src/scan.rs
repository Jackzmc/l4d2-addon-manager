use std::collections::{HashSet, VecDeque};
use std::fmt::Display;
use std::iter::Scan;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use chrono::{DateTime, Utc};
use l4d2_addon_parser::{AddonInfo, L4D2Addon};
use log::{debug, error, info, trace};
use regex::Regex;
use serde::Serialize;
use steam_workshop_api::{SteamWorkshop, WorkshopItem};
use tauri::{AppHandle, Emitter};
use tokio::{join, task};
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use crate::addons::{AddonData, AddonFlags, AddonStorageContainer};
use crate::scan::ScanError::{DBError, FileError, NewEntryError, ParseError, UpdateError, UpdateRenameError};

pub struct AddonScanner {
    scan_thread: Option<JoinHandle<()>>,
    abort_signal: Arc<AtomicBool>,

    addons: AddonStorageContainer,
    app: AppHandle
}

pub type ScannerContainer = Mutex<AddonScanner>;
type ScanQueue = Arc<Mutex<VecDeque<PathBuf>>>;

const NUM_WORKER_THREADS: usize = 1;

impl AddonScanner {
    pub fn new(addons: AddonStorageContainer, app: AppHandle) -> Self {
        Self {
            scan_thread: None,
            abort_signal: Arc::new(AtomicBool::new(false)),
            addons,
            app
        }
    }

    /// Starts an async background scan. New items will appear in database on their own
    /// The scan starts a main thread that first scans both addons/ and addons/workshop dirs
    /// All addons/*.vpk are in a queue to be processed in N (NUM_THREADS) worker threads
    /// When worker threads done, any new workshop IDs are returned, and merged into main thread's workshop ids
    /// Workshop ids then are resolved in batches of 100 (steam's max limit)
    pub fn start_scan(&mut self, path: PathBuf) -> bool {
        if !self.is_running() {
            let addons = self.addons.clone();
            let app = self.app.clone();
            let abort_signal = self.abort_signal.clone();

            self.scan_thread = Some(std::thread::Builder::new().name("scan_main_thread".to_string()).spawn( || scan_main_thread(path, abort_signal, addons, app)).unwrap());
        }
        true
    }
    pub fn abort_scan(&mut self) {
        if !self.is_running() { return; } // ignore if not running

        self.abort_signal.store(true, Ordering::SeqCst);

        info!("Waiting for threads to end");

        info!("Scan aborted");
    }

    /// Is a scan running
    pub fn is_running(&self) -> bool {
        self.scan_thread.is_some()
    }
}
#[derive(Default, Clone)]
struct ScanCounter {
    total: Arc<AtomicU32>,
    added: Arc<AtomicU32>,
    errors: Arc<AtomicU32>
}

/// Performs a scan of directory returning list of pathbufs
fn get_vpks_in_dir(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    info!("Scanning addons at {}", path.display());
    let dir = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for file in dir {
        let file = file.map_err(|e| e.to_string())?;
        let path = file.path();
        if let Some(ext) = path.extension() {
            if ext == "vpk" {
                list.push(path);
            }
        }
    }
    Ok(list)
}

/// Main thread that starts and manages thread
fn scan_main_thread(path: PathBuf, abort_signal: Arc<AtomicBool>,addons: AddonStorageContainer, app: AppHandle) {
    let counter = Arc::new(ScanCounter::default());
    app.emit("scan_state", ScanState::Started).ok();

    // Load queue with vpks before starting worker threads
    let mut queue = VecDeque::new();
    let files = get_vpks_in_dir(&path).expect("failed to scan dir");
    for vpk in files  {
        queue.push_front(vpk);
    }

    // Extract all workshop ids from workshop addons folder
    let mut ws_addons: Vec<i64> = get_vpks_in_dir(&path.join("workshop"))
        .expect("failed to scan ws dir")
        .into_iter()
        .map(|item| item.file_stem().unwrap().to_string_lossy().parse::<i64>())
        // Remove any files that don't have a valid ID:
        .filter(|item| item.is_ok())
        .map(|item| item.unwrap())
        .collect();


    // Allow aborting early right before we enter the main process loop
    if abort_signal.load(Ordering::SeqCst) {
        info!("Got early abort signal, ending");
        return;
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_name("scan-worker")
        .enable_time()
        .max_blocking_threads(1)
        .worker_threads(NUM_WORKER_THREADS)
        .build()
        .expect("could not build worker runtime");

    info!("Spawning {} worker tasks", NUM_WORKER_THREADS);
    let mut set = JoinSet::new();
    // Drain the queue and start a task for every item
    while let Some(item) = queue.pop_back() {
        let addons = addons.clone();
        let counter = counter.clone();
        set.spawn_on(scan_file_wrapper(item, addons, counter), rt.handle());
    }

    // The following section is a bit hacky - I'm not sure how to properly run this
    // as it needs to be .await'd, but we are running on a sync thread
    // so there is a task running on a task thread and this main thread is just sleeping
    let (tx, rx) = channel::<()>();
    let ws = Arc::new(SteamWorkshop::new());

    // Spawn a task to await all the other worker tasks
    rt.spawn(async move {
        let mut is_aborting = false;
        debug!("scan main: waiting for tasks to finish");
        while let Some(Ok(result)) = set.join_next().await {
            match result {
                WorkerOutput::WorkshopId(workshop_id) => {
                    ws_addons.push(workshop_id);
                    // If we got >= 100 workshop ids, while we are still processing, go ahead and fetch them
                    if ws_addons.len() >= 100 {
                        start_workshop_resolve_task(&addons, &mut ws_addons, &mut set, &ws);
                    }
                },
                _ => {}
            }

            // Check if we should abort, only abort once
            if !is_aborting && abort_signal.load(Ordering::SeqCst) {
                set.abort_all();
                is_aborting = true;
            }
        }
        debug!("all tasks complete");
        // All tasks complete, resolve any remaining workshop ids
        while ws_addons.len() > 0 {
            start_workshop_resolve_task(&addons, &mut ws_addons, &mut set, &ws);
        }

        info!("Scan completed, fetching work items");
        info!("Workshop Items to Process: {}", &ws_addons.len());
        // TODO: process!
        info!("Workshop item fetching complete");

        tx.send(()).unwrap();
    });
    drop(rt);

    // Wait until wait task signals it's completion
    debug!("scan main thread sleeping");
    rx.recv().ok();
    app.emit("scan_state", ScanState::Complete).ok();
    info!("ADDON SCAN COMPLETE. {} addons scanned, {} added, {} failed", counter.total.load(Ordering::SeqCst), counter.added.load(Ordering::SeqCst), counter.errors.load(Ordering::SeqCst));
}

/// Takes upto 100 ids and spawns new fetch task
/// Runs on tokio main worker threads to avoid scan aborts dropping it
fn start_workshop_resolve_task(addons: &AddonStorageContainer, ws_addons: &mut Vec<i64>, set: &mut JoinSet<WorkerOutput>, ws: &Arc<SteamWorkshop>) {
    // Steam API only supports upto 100 at a time
    let items_to_drain = 100.min(ws_addons.len()); // drain panics if over len, get smallest
    let slice: Vec<String> = ws_addons.drain(0..items_to_drain).map(|item| item.to_string()).collect();

    let addons = addons.clone();
    let ws = ws.clone();
    trace!("spawning workshop task");
    task::spawn(get_workshop_ids(ws, slice, addons));
}

enum WorkerOutput {
    /// Worker has new workshop id to enqueue
    WorkshopId(i64),
    /// Worker has fetched workshop items
    WorkshopItems(Vec<WorkshopItem>),
    /// Worker has nothing useful
    None
}

async fn get_workshop_ids(ws: Arc<SteamWorkshop>, slice: Vec<String>, addons: AddonStorageContainer) -> WorkerOutput {
    info!("Fetching {} workshop ids", slice.len());
    if let Ok(items) = ws.get_published_file_details(&slice) {
        let addons = addons.lock().await;
        addons.add_workshop_items(items).await
            .expect("failed to add workshop items");
    }
    WorkerOutput::None
}

/// Parallel worker thread(s)
// fn scan_worker_thread(queue: ScanQueue, addons: AddonStorageContainer, counter: Arc<ScanCounter>, app: AppHandle) {
//     let mut workshop_ids: Vec<i64> = Vec::new();
//     loop {
//         let item = {
//             let mut queue = queue.lock().unwrap();
//             if queue.len() == 0 { break; }
//             queue.pop_back().unwrap()
//         };
//
//         let addons = addons.clone();
//         let counter = counter.clone();
//         let app = app.clone();
//         tauri::async_runtime::block_on(async move {
//             counter.total.fetch_add(1, Ordering::Relaxed);
//             let filename = item.file_name().unwrap().to_string_lossy().to_string();
//             match scan_file(item, addons).await {
//                 Ok(result) => {
//                     match result {
//                         ScanResult::Added => {
//                             counter.added.fetch_add(1, Ordering::Relaxed);
//                         },
//                         _ => {}
//                     };
//                     app.emit("scan_result", ScanResultPayload {
//                         result,
//                         filename
//                     }).ok();
//                 },
//                 Err(e) => {
//                     error!("SCAN ERROR FOR \"{}\": {}", filename, e);
//                     counter.errors.fetch_add(1, Ordering::Relaxed);
//                 },
//             }
//         });
//     }
// }
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
async fn scan_file_wrapper(path: PathBuf, addons: AddonStorageContainer, counter: Arc<ScanCounter>) -> WorkerOutput {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    counter.total.fetch_add(1, Ordering::Relaxed);
    match scan_file(path, addons).await {
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
async fn scan_file(path: PathBuf, addons: AddonStorageContainer) -> Result<(ScanResult, Option<AddonData>), ScanError> {
    let meta = path.metadata().map_err(|e| FileError(e))?;
    let filename = path.file_name().unwrap().to_str().unwrap();
    let addon_entry = {
        let addons = addons.lock().await;
        addons.get_by_filename(filename).await
            .map_err(|e| DBError(e))?
    };

    let (info, chapter_ids) = parse_addon(&path).await
        .map_err(|e| ParseError(e))?;

    if let Some(entry) = addon_entry {
        let last_modified = meta.modified().map_err(|e| FileError(e))?;
        // Check if file has been modified since scanned
        if <DateTime<Utc>>::from(last_modified) > entry.updated_at {
            let mut addons = addons.lock().await;
            debug!("file has changed, updating entry {:?}", path);
            addons.update_entry(filename, meta, info).await
                .map_err(|e| UpdateError(e))?;
            return Ok((ScanResult::Updated, None))
        }
        return Ok((ScanResult::NoAction, None))
    }

    let mut addons = addons.lock().await;
    // If info has title and version, try to find previous entry and update its filename
    if let Some(title) = &info.title && let Some(version) = &info.version {
        // If we found a previous entry, we are done.
        // Next time a scan is performed any changes will be reflected by the last modified check
        if addons.update_entry_pk(filename, version, title).await
            .map_err(|e| UpdateRenameError(e))?
        {
            return Ok((ScanResult::Renamed, None))
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

    // Add to DB
    addons.add_entry(&data).await
        .map_err(|e| NewEntryError(e))?;

    Ok((ScanResult::Added, Some(data)))
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
            return Some(id.parse::<i64>().unwrap());
        }
    }

    // Try to get it from filename
    let filename = path.file_name().unwrap().to_str().unwrap();
    if let Some(cap) = WORKSHOP_FILE_REGEX.find(filename) {
        let id = cap.as_str().parse::<i64>().unwrap();
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
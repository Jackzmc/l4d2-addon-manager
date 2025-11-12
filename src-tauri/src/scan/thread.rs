use crate::scan::helpers::get_vpks_in_dir;
use crate::scan::worker::scan_file_wrapper;
use crate::scan::worker::WorkerOutput;
use crate::scan::ScanCounter;
use crate::scan::ScanState;
use crate::modules::store::AddonStorageContainer;
use log::debug;
use log::error;
use log::info;
use log::trace;
use rand::random;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Instant;
use steam_workshop_api::SteamWorkshop;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::runtime::Handle;
use tokio::task::JoinSet;

const NUM_WORKER_THREADS: usize = 4;

fn get_workshop_folder_ws_ids(path: &PathBuf) -> Vec<i64> {
    match get_vpks_in_dir(&path.join("workshop")) {
        Ok(list) => list.into_iter()
            .map(|item| item.file_stem().unwrap().to_string_lossy().parse::<i64>())
            // Remove any files that don't have a valid ID:
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect(),
        Err(e) => {
            error!("failed to scan workshop dir: {}", e);
            Vec::new()
        }
    }
}

/// Main thread that starts and manages thread
pub(super) fn scan_main_thread(path: PathBuf, running_signal: Arc<AtomicBool>, addons: AddonStorageContainer, app: AppHandle) {
    let counter = Arc::new(ScanCounter::default());
    app.emit("scan_state", ScanState::Started).ok();
    let scan_id: u32 = random();
    info!("=== SCAN STARTED ===");
    info!("workers={} scan_id={}", NUM_WORKER_THREADS, scan_id);
    info!("====================");
    let now = Instant::now();

    // Load queue with vpks before starting worker threads
    let mut files = get_vpks_in_dir(&path).expect("failed to scan dir");

    // Allow aborting early right before we enter the main process loop
    if !running_signal.load(Ordering::SeqCst) {
        info!("Got early abort signal, ending");
        return;
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_name("scan-worker")
        .enable_time()
        .max_blocking_threads(NUM_WORKER_THREADS)
        .worker_threads(NUM_WORKER_THREADS)
        .build()
        .expect("could not build worker runtime");

    let mut set = JoinSet::new();
    // Drain the queue and start a task for every item
    while let Some(item) = files.pop() {
        let addons = addons.clone();
        let counter = counter.clone();
        set.spawn_on(scan_file_wrapper(item, addons, counter, scan_id), rt.handle());
    }

    // The following section is a bit hacky - I'm not sure how to properly run this
    // as it needs to be .await'd, but we are running on a sync thread
    // so there is a task running on a task thread and this main thread is just sleeping
    let (tx, rx) = channel::<()>();
    let ws = Arc::new(SteamWorkshop::new());
    let task_running_signal = running_signal.clone();
    let handle = rt.handle().clone();

    // Spawn a task to await all the other worker tasks
    rt.spawn(async move {
        // Remove any workshop ids from workshop/ folder that we already got:
        info!("Filtering out existing workshop entries");
        let existing_ws_ids = {
            let addons = addons.lock().await;
            addons.list_workshop_ids().await.unwrap_or_default()
        };

        resolve_workshop_folder(&path, &addons, &ws, &handle, existing_ws_ids).await;

        // Addons from the normal addons folder, their workshop ids get added to this queue
        let mut addons_folder_ws_ids: Vec<i64> = Vec::new();

        let mut is_aborting = false;
        debug!("scan main: waiting for tasks to finish");
        while let Some(Ok(result)) = set.join_next().await {
            match result {
                WorkerOutput::WorkshopId(workshop_id) => {
                    addons_folder_ws_ids.push(workshop_id);
                    // If we got >= 100 workshop ids, while we are still processing, go ahead and fetch them
                    if addons_folder_ws_ids.len() >= 100 {
                        drain_workshop_addons(&mut addons_folder_ws_ids, &handle, &ws, addons.clone()).await;
                    }
                },
                _ => {}
            }

            // Check if we should abort, only abort once
            // running_signal is set true initially, if it's false, we should abort
            if !is_aborting && !task_running_signal.load(Ordering::Relaxed) {
                set.abort_all();
                is_aborting = true;
            }
        }
        debug!("all addon scan tasks complete");
        // All tasks complete, resolve any remaining workshop ids
        drain_workshop_addons(&mut addons_folder_ws_ids, &handle, &ws, addons.clone()).await;

        // Mark all filenames and workshop srcs as null if we did not update / add them in this scan
        let addons = addons.lock().await;
        addons.scan_mark_missing(scan_id).await.unwrap();

        info!("all tasks done");

        tx.send(()).unwrap();
    });

    // Wait until wait task signals it's completion
    trace!("main thread sleeping");
    if let Err(e) = rx.recv() {
        debug!("{}", e);
    }
    app.emit("scan_state", ScanState::Complete {
        total: counter.total.load(Ordering::SeqCst),
        added: counter.added.load(Ordering::SeqCst),
        failed: counter.errors.load(Ordering::SeqCst)
    }).ok();

    info!("===SCAN COMPLETE===");
    info!("{} addons scanned, {} added, {} failed", counter.total.load(Ordering::SeqCst), counter.added.load(Ordering::SeqCst), counter.errors.load(Ordering::SeqCst));
    info!("Duration: {} seconds", now.elapsed().as_secs());
    info!("===================");
    running_signal.store(true, Ordering::SeqCst); // signal that scan over
}

async fn resolve_workshop_folder(path: &PathBuf, addons: &AddonStorageContainer, ws: &Arc<SteamWorkshop>, handle: &Handle, existing_ws_ids: Vec<i64>) {
    // Extract all workshop ids from workshop addons folder
    // Fetch any items that we don't already have
    let mut workshop_folder_ws_ids: Vec<i64> = get_workshop_folder_ws_ids(&path).into_iter().filter(|id| !existing_ws_ids.contains(id)).collect();
    let workshop_ids_clone = workshop_folder_ws_ids.clone();
    // If there is any workshop ids in the workshop folder, fetch them
    drain_workshop_addons(&mut workshop_folder_ws_ids, &handle, &ws, addons.clone()).await;

    // Then mark all the items in workshop folder as workshop items
    let addons = addons.lock().await;
    if let Err(e) = addons.mark_workshop_ids(workshop_ids_clone).await {
        error!("failed to mark workshop ids: {}", e);
    }
}

/// Drains list of workshop ids and fetches them in batches of 100.
/// Runs on runtime blocking threads as it's sync HTTP
async fn drain_workshop_addons(addon_ids: &mut Vec<i64>, rt: &Handle, ws: &Arc<SteamWorkshop>, addons: AddonStorageContainer) {
    while !addon_ids.is_empty() {
        fetch_workshop_addons(addon_ids, rt, ws, addons.clone()).await;
    }
}

/// takes upto 100 ids from list and fetches items and pushes to db
async fn fetch_workshop_addons(addon_ids: &mut Vec<i64>, rt: &Handle, ws: &Arc<SteamWorkshop>, addons: AddonStorageContainer) {
    // Steam API only supports upto 100 at a time
    let items_to_drain = 100.min(addon_ids.len()); // drain panics if over len, get smallest
    trace!("fetching slice of {items_to_drain} ids");
    let slice: Vec<String> = addon_ids.drain(0..items_to_drain).map(|item| item.to_string()).collect();
    let ws = ws.clone();
    let slice_len = slice.len();
    match rt.spawn_blocking(move || ws.get_published_file_details(&slice)).await.expect("failed to spawn blocking task") {
        Ok(items) => {
            if items.len() == 0 { return; } // skip if we have nothing
            let addons = addons.lock().await;
            debug!("fetched {} ids, got {} workshop items", slice_len, items.len());
            if let Err(err) = addons.add_workshop_items(items).await {
                error!("failed to add workshop items: {}", err);
            }
        },
        Err(err) => {
            error!("failed to get workshop ids: {}", err);
        }
    }
}
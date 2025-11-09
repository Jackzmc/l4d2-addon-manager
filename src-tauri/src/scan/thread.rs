use crate::scan::ScanState;
use steam_workshop_api::WorkshopItem;
use tokio::runtime::Handle;
use crate::scan::worker::WorkerOutput;
use steam_workshop_api::SteamWorkshop;
use crate::scan::worker::scan_file_wrapper;
use tokio::task::JoinSet;
use std::sync::atomic::Ordering;
use crate::scan::helpers::get_vpks_in_dir;
use std::collections::VecDeque;
use crate::scan::ScanCounter;
use tauri::AppHandle;
use crate::store::AddonStorageContainer;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Instant;
use log::trace;
use log::debug;
use log::info;
use log::error;
use rand::random;
use tauri::Emitter;

const NUM_WORKER_THREADS: usize = 4;

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
    let mut queue = VecDeque::new();
    let files = get_vpks_in_dir(&path).expect("failed to scan dir");
    for vpk in files  {
        queue.push_front(vpk);
    }

    // Extract all workshop ids from workshop addons folder
    let ws_addons: Vec<i64> = get_vpks_in_dir(&path.join("workshop"))
        .expect("failed to scan ws dir")
        .into_iter()
        .map(|item| item.file_stem().unwrap().to_string_lossy().parse::<i64>())
        // Remove any files that don't have a valid ID:
        .filter(|item| item.is_ok())
        .map(|item| item.unwrap())
        .collect();

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
    while let Some(item) = queue.pop_back() {
        let addons = addons.clone();
        let counter = counter.clone();
        set.spawn_on(scan_file_wrapper(item, addons, counter, scan_id), rt.handle());
    }

    // The following section is a bit hacky - I'm not sure how to properly run this
    // as it needs to be .await'd, but we are running on a sync thread
    // so there is a task running on a task thread and this main thread is just sleeping
    let (tx, rx) = channel::<()>();
    let ws = Arc::new(SteamWorkshop::new());

    // Spawn a task to await all the other worker tasks
    let task_running_signal = running_signal.clone();
    let handle = rt.handle().clone();
    rt.spawn(async move {
        // Remove any workshop ids from workshop/ folder that we already got:
        info!("Filtering out existing workshop entries");
        let existing_ws_ids = {
            let addons = addons.lock().await;
            addons.list_workshop_ids().await.unwrap_or_default()
        };
        let mut ws_addons: Vec<i64> = ws_addons.into_iter().filter(|id| !existing_ws_ids.contains(id)).collect();

        let mut is_aborting = false;
        debug!("scan main: waiting for tasks to finish");
        while let Some(Ok(result)) = set.join_next().await {
            match result {
                WorkerOutput::WorkshopId(workshop_id) => {
                    ws_addons.push(workshop_id);
                    // If we got >= 100 workshop ids, while we are still processing, go ahead and fetch them
                    if ws_addons.len() >= 100 {
                        start_workshop_resolve_task(&mut ws_addons, &handle, &ws);
                    }
                },
                WorkerOutput::WorkshopItems(items ) => add_workshop(&addons, items).await,
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
        while ws_addons.len() > 0 {
            if let WorkerOutput::WorkshopItems(items) = start_workshop_resolve_task(&mut ws_addons, &handle, &ws).await.unwrap() {
                add_workshop(&addons, items).await;
            }
        }

        // Mark all filenames as null if we did not update / add them in this scan
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

/// Takes upto 100 ids and spawns new fetch task
/// Runs on tokio main worker threads to avoid scan aborts dropping it
fn start_workshop_resolve_task(ws_addons: &mut Vec<i64>, rt: &Handle, ws: &Arc<SteamWorkshop>) -> tokio::task::JoinHandle<WorkerOutput> {
    // Steam API only supports upto 100 at a time
    let items_to_drain = 100.min(ws_addons.len()); // drain panics if over len, get smallest
    let slice: Vec<String> = ws_addons.drain(0..items_to_drain).map(|item| item.to_string()).collect();
    let ws = ws.clone();
    trace!("spawning workshop task");
    rt.spawn_blocking(|| get_workshop_ids(ws, slice))
}

async fn add_workshop(addons: &AddonStorageContainer, items: Vec<WorkshopItem>) {
    if items.len() == 0 { return; }
    debug!("got {} items to store", items.len());
    let addons = addons.lock().await;
    addons.add_workshop_items(items).await
        .expect("failed to add workshop items");
    debug!("stored");
}

fn get_workshop_ids(ws: Arc<SteamWorkshop>, slice: Vec<String>) -> WorkerOutput {
    info!("Fetching {} workshop ids ({:?})", slice.len(), slice);
    match ws.get_published_file_details(&slice) {
        Ok(items) => {
            WorkerOutput::WorkshopItems(items)
        },
        Err(err) => {
            error!("failed to get workshop ids: {}", err);
            WorkerOutput::None
        }
    }
}
use crate::modules::store::AddonStorageContainer;
use crate::scan::{ScanProgress, ScanState};
use crate::scan::helpers::{get_vpks_in_dir, get_workshop_folder_ws_ids};
use crate::scan::worker::{
    AddonFileData, ProcessResult, WorkerTask, async_process_file, scan_worker_thread,
    scan_workshop_thread,
};
use crate::scan::{ScanCounter, ScanSpeed};
use log::error;
use log::info;
use log::trace;
use log::{debug, warn};
use rand::random;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tauri::AppHandle;
use tauri::Emitter;

/// Main thread that starts and manages thread
pub(super) async fn scan_main(
    path: PathBuf,
    speed: ScanSpeed,
    running_signal: Arc<AtomicBool>,
    addons: AddonStorageContainer,
    app: AppHandle,
) {
    let threads = speed.threads();
    let mut counter = ScanCounter::default();
    app.emit("scan_state", ScanState::Started { speed }).ok();
    let scan_id: u32 = random();
    info!("===== SCAN STARTED =====");
    info!("speed={} scan_id={}", speed, scan_id);
    info!("========================");
    let now = Instant::now();

    // Fetch addons and start worker threads
    let scan_tasks: Vec<WorkerTask> = get_vpks_in_dir(&path)
        .expect("failed to scan dir")
        .into_iter()
        .map(|path| WorkerTask::ScanFile(path))
        .collect();
    let items_to_scan = scan_tasks.len() as u32;

    // Allow aborting early right before we enter the main process loop
    if !running_signal.load(Ordering::SeqCst) {
        info!("Got early abort signal (1), ending");
        return;
    }

    debug!("scan_main: got {} files to scan", scan_tasks.len());
    // queue being empty signals threads to end
    let queue = Arc::new(tokio::sync::Mutex::new(VecDeque::<WorkerTask>::from(
        scan_tasks,
    )));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<AddonFileData, String>>(60);
    // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<AddonFileData, String>>();

    debug!("starting {} worker threads", threads);
    for i in 0..threads {
        let tx = tx.clone();
        let queue = queue.clone();
        std::thread::Builder::new()
            .name("scan-worker-thread".to_string())
            .spawn(move || scan_worker_thread(i as u8, tx, queue))
            .expect("failed to spawn worker thread");
    }
    drop(tx); // we don't use it, need to drop so we don't hang

    // acquiring all workshop ids so we can skip fetching any items we have
    debug!("getting existing workshop ids");
    let existing_ws_ids = {
        let addons = addons.lock().await;
        addons.list_workshop_ids().await.unwrap_or_default()
    };

    // Process results of worker threads
    let mut workshop_ids: Vec<i64> = Vec::new();
    trace!("starting scan file processing loop");
    while let Some(result) = rx.recv().await {
        match result {
            Ok(file) => {
                counter.total += 1;
                match async_process_file(file, addons.clone(), scan_id).await {
                    Ok((ProcessResult::Added, workshop_id)) => {
                        counter.added += 1;
                        if let Some(ws_id) = workshop_id {
                            if !existing_ws_ids.contains(&ws_id) {
                                workshop_ids.push(ws_id);
                            }
                        }
                    }
                    Ok((ProcessResult::UpdatedByHash, _) | (ProcessResult::UpdatedByFilename, _)) => {
                        counter.updated += 1;
                    }
                    Err(err) => {
                        counter.errors += 1;
                        error!("process_file: {}", err);
                    }
                }
            }
            Err(err) => {
                warn!("scan_file: {}", err);
            }
        };

        app.emit("scan_progress", ScanProgress { items: items_to_scan, processed: counter.total }).ok();

        // Check if we should abort
        if !running_signal.load(Ordering::SeqCst) {
            info!("Got abort signal in process loop, ending");
            queue.lock().await.clear(); // drain queue to signal worker threads to end
            while let Some(_) = rx.recv().await {} // wait for all threads to end
            return;
        }
    }
    debug!("all addons scanned and processed");

    debug!("resolving workshop folder addons");
    let workshop_folder_ids = get_workshop_folder_ws_ids(&path);
    // merge any missing workshop folder ids to queue
    workshop_ids.extend(
        workshop_folder_ids
            .iter()
            .filter(|id| !existing_ws_ids.contains(id)),
    );
    let workshop_items = std::thread::spawn(|| scan_workshop_thread(workshop_ids))
        .join()
        .expect("workshop thread panicked");

    let addons = addons.lock().await;
    debug!("adding {} workshop items", workshop_items.len());
    addons
        .add_workshop_items(workshop_items)
        .await
        .expect("failed to add workshop items");
    debug!("marking {} workshop ids", workshop_folder_ids.len());
    addons
        .mark_workshop_ids(workshop_folder_ids)
        .await
        .expect("failed to mark workshop ids"); // this should be after add_workshop_items, need items to exist first
    debug!("marking any missing files");
    addons
        .scan_mark_missing(scan_id)
        .await
        .expect("failed to mark missing files");

    info!("all tasks done");

    app.emit(
        "scan_state",
        ScanState::Complete {
            time: now.elapsed().as_secs(),
            total: counter.total,
            added: counter.added,
            updated: counter.updated,
            failed: counter.errors,
        },
    )
    .ok();

    info!("====== SCAN COMPLETE ======");
    info!(
        "{} addons scanned, {} added, {} updated, {} failed",
        counter.total, counter.added, counter.updated, counter.errors,
    );
    info!("Duration: {} seconds", now.elapsed().as_secs());
    info!("===========================");
    running_signal.store(true, Ordering::SeqCst); // signal that scan over
}

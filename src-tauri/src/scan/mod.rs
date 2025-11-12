use std::sync::atomic::Ordering;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use crate::modules::store::AddonStorageContainer;
use std::sync::atomic::AtomicBool;
use crate::scan::worker::ProcessError;
use std::fmt::Display;
use serde::Serialize;
use log::info;
use log::debug;
use std::sync::Arc;
use tauri::Emitter;
use crate::scan::thread::scan_main;

mod helpers;
mod thread;
mod worker;

#[derive(Default)]
struct ScanCounter {
    total: u32,
    added: u32,
    updated: u32,
    errors: u32
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state")]
pub enum ScanState {
    Started,
    Aborted {
        reason: Option<String>
    },
    Complete {
        total: u32,
        added: u32,
        updated: u32,
        failed: u32
    }
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::FileError(e) => write!(f, "IO Error: {}", e),
            ProcessError::UpdateExistingError(e) => write!(f, "Error updating existing item: {}", e),
            ProcessError::NewEntryError(e) => write!(f, "Error creating new entry: {}", e),
        }
    }
}

pub struct AddonScanner {
    scan_main_task: Option<tokio::task::JoinHandle<()>>,
    running_signal: Arc<AtomicBool>,
    addons: AddonStorageContainer,
    app: AppHandle
}

pub type ScannerContainer = Mutex<AddonScanner>;
impl AddonScanner {
    pub fn new(addons: AddonStorageContainer, app: AppHandle) -> Self {
        Self {
            scan_main_task: None,
            running_signal: Arc::new(AtomicBool::new(false)),
            addons,
            app
        }
    }

    /// Starts an async background scan. New items will appear in database on their own
    /// The scan starts a main thread that first scans both addons/ and addons/workshop dirs
    /// All addons/*.vpk have a task spawned in (NUM_WORKER_THREADS) task threads
    /// When worker tasks complete, any workshop ids to fetch items are sent, and resolved once we have over 100
    /// When all worker tasks done, any remaining workshop items are fetched in batches of 100
    pub fn start(&mut self, path: PathBuf) -> bool {
        if self.check_running() { return false; } // ignore if not running

        let addons = self.addons.clone();
        let app = self.app.clone();
        let running_signal = self.running_signal.clone();
        self.running_signal.store(true, Ordering::SeqCst);
        self.scan_main_task = Some(tokio::spawn(scan_main(path, running_signal, addons, app)));
        true
    }
    pub fn abort(&mut self, reason: Option<String>) {
        if !self.check_running() { return; } // ignore if not running

        // this tells thread to abort, but reusing the same signal does
        self.running_signal.store(false, Ordering::SeqCst);
        // wait for thread to end
        let main_task = self.scan_main_task.take().unwrap();
        while !main_task.is_finished() {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        info!("Scan aborted");
        self.app.emit("scan_state", ScanState::Aborted {
            reason
        }).ok();
    }

    /// Is a scan running
    pub fn check_running(&mut self) -> bool {
        self._check_thread_complete()
    }

    /// Checks if we still have a thread handle, checks if thread finished, and removes ref
    fn _check_thread_complete(&mut self) -> bool {
        if let Some(thread) = self.scan_main_task.as_ref() {
            debug!("thread still exists, check");
            if thread.is_finished() {
                debug!("its finished, removing it");
                self.scan_main_task.take();
                return false
            }
            return true
        }
        false
    }
}
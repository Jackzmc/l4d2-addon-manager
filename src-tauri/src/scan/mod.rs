use crate::scan::thread::scan_main_thread;
use std::sync::atomic::Ordering;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use crate::modules::store::AddonStorageContainer;
use std::sync::atomic::AtomicBool;
use crate::scan::worker::ScanError;
use std::fmt::Display;
use serde::Serialize;
use log::info;
use log::debug;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use tauri::Emitter;

mod helpers;
mod thread;
mod worker;

#[derive(Default, Clone)]
struct ScanCounter {
    total: Arc<AtomicU32>,
    added: Arc<AtomicU32>,
    updated: Arc<AtomicU32>,
    errors: Arc<AtomicU32>
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

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::FileError(e) => write!(f, "IO Error: {}", e),
            ScanError::ParseError(e) => write!(f, "Parse Error: {}", e),
            ScanError::UpdateExistingError(e) => write!(f, "Error updating existing item: {}", e),
            ScanError::NewEntryError(e) => write!(f, "Error creating new entry: {}", e),
        }
    }
}

pub struct AddonScanner {
    scan_thread: Option<std::thread::JoinHandle<()>>,
    running_signal: Arc<AtomicBool>,

    addons: AddonStorageContainer,
    app: AppHandle
}

pub type ScannerContainer = Mutex<AddonScanner>;
impl AddonScanner {
    pub fn new(addons: AddonStorageContainer, app: AppHandle) -> Self {
        Self {
            scan_thread: None,
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

        self.scan_thread = Some(std::thread::Builder::new().name("scan_main_thread".to_string())
            .spawn( || scan_main_thread(path, running_signal, addons, app)).unwrap());
        true
    }
    pub fn abort(&mut self, reason: Option<String>) {
        if !self.check_running() { return; } // ignore if not running

        // this tells thread to abort, but reusing the same signal does
        self.running_signal.store(false, Ordering::SeqCst);
        // wait for thread to end
        self.scan_thread.take().unwrap().join().unwrap();
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
        if let Some(thread) = self.scan_thread.as_ref() {
            debug!("thread still exists, check");
            if thread.is_finished() {
                debug!("its finished, removing it");
                self.scan_thread.take();
                return false
            }
            return true
        }
        false
    }
}
use crate::modules::store::AddonStorageContainer;
use crate::scan::thread::scan_main;
use crate::scan::worker::ProcessError;
use log::debug;
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::__rt::timeout;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::async_runtime::block_on;
use tokio::sync::Mutex;

mod helpers;
mod thread;
mod worker;

const SCAN_ABORT_TIMEOUT_SEC: u64 = 60;

#[derive(Default)]
struct ScanCounter {
    total: u32,
    added: u32,
    updated: u32,
    errors: u32,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ScanProgress {
    items: u32,
    processed: u32
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state")]
pub enum ScanState {
    Started {
        speed: ScanSpeed,
    },
    Aborted {
        reason: Option<String>,
    },
    Complete {
        time: u64,
        total: u32,
        added: u32,
        updated: u32,
        failed: u32,
    },
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::FileError(e) => write!(f, "IO Error: {}", e),
            ProcessError::UpdateExistingError(e) => {
                write!(f, "Error updating existing item: {}", e)
            }
            ProcessError::NewEntryError(e) => write!(f, "Error creating new entry: {}", e),
        }
    }
}

pub struct AddonScanner {
    scan_main_task: Option<tokio::task::JoinHandle<()>>,
    running_signal: Arc<AtomicBool>,
    addons: AddonStorageContainer,
    app: AppHandle,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ScanSpeed {
    /// Uses all threads
    Maximum,
    /// Uses half of threads
    Normal,
    /// Uses one thread
    Background,
}

impl Display for ScanSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let threads = self.threads();
        match self {
            ScanSpeed::Maximum => write!(f, "Maximum ({} threads)", threads),
            ScanSpeed::Normal => write!(f, "Normal ({} threads)", threads),
            ScanSpeed::Background => write!(f, "Background ({} threads)", threads),
        }
    }
}

impl Default for ScanSpeed {
    fn default() -> Self {
        ScanSpeed::Normal
    }
}

impl ScanSpeed {
    /// Returns number of threads to use based on scan speed setting
    pub fn threads(&self) -> u8 {
        match self {
            ScanSpeed::Maximum => num_cpus::get() as u8,
            ScanSpeed::Normal => (num_cpus::get() as f32 / 2.0).ceil() as u8,
            ScanSpeed::Background => 1,
        }
    }
}

pub type ScannerContainer = Mutex<AddonScanner>;
impl AddonScanner {
    pub fn new(addons: AddonStorageContainer, app: AppHandle) -> Self {
        Self {
            scan_main_task: None,
            running_signal: Arc::new(AtomicBool::new(false)),
            addons,
            app,
        }
    }

    /// Starts an async background scan. New items will appear in database on their own
    /// The scan starts a main thread that first scans both addons/ and addons/workshop dirs
    /// All addons/*.vpk have a task spawned in (NUM_WORKER_THREADS) task threads
    /// When worker tasks complete, any workshop ids to fetch items are sent, and resolved once we have over 100
    /// When all worker tasks done, any remaining workshop items are fetched in batches of 100
    pub fn start(&mut self, path: PathBuf, speed: ScanSpeed) -> bool {
        if self.check_running() {
            return false;
        } // ignore if not running

        let addons = self.addons.clone();
        let app = self.app.clone();
        let running_signal = self.running_signal.clone();
        self.running_signal.store(true, Ordering::SeqCst);
        self.scan_main_task = Some(tokio::spawn(scan_main(
            path,
            speed,
            running_signal,
            addons,
            app,
        )));
        true
    }
    pub async fn abort(&mut self, mut reason: Option<String>) {
        if !self.check_running() {
            return;
        } // ignore if not running

        // this tells thread to abort, but reusing the same signal does
        self.running_signal.store(false, Ordering::SeqCst);
        // wait for thread to end
        // let main_task = self.scan_main_task.take().unwrap();
        let abort_timed = timeout(
            Duration::from_secs(SCAN_ABORT_TIMEOUT_SEC),
            self.scan_main_task.take().unwrap(),
        )
        .await
        .is_err();
        if abort_timed {
            reason = reason.map(|reason| format!("{} (timed out)", reason));
        }
        info!("Scan aborted for \"{:?}\"", reason);
        self.app
            .emit("scan_state", ScanState::Aborted { reason })
            .ok();
    }

    /// Is a scan running
    pub fn check_running(&mut self) -> bool {
        self._check_thread_complete()
    }

    /// Checks if we still have a thread handle, checks if thread finished, and removes ref
    fn _check_thread_complete(&mut self) -> bool {
        if let Some(task) = self.scan_main_task.as_ref() {
            debug!("scan task still exists, checking if its finished");
            if task.is_finished() {
                debug!("its finished, removing it");
                self.scan_main_task.take();
                return false;
            }
            return true;
        }
        false
    }
}

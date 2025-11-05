use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use chrono::{DateTime, Utc};
use l4d2_addon_parser::{AddonInfo, L4D2Addon, MissionInfo};
use log::{debug, info};
use tauri::async_runtime::{channel, Receiver, Sender};
use crate::addons::AddonStorageContainer;

pub struct AddonScanner {
    tx: Sender<PathBuf>,
    thread: Option<JoinHandle<()>>,

    addons: Arc<AddonStorageContainer>
}

impl AddonScanner {
    // fn _scan(&self, path: PathBuf) {
    //     info!("Scanning addons at {}", path.display());
    //     let dir = std::fs::read_dir(path)
    //         .map_err(|e| e.to_string())?;
    //     for file in dir {
    //         let file = file.map_err(|e| e.to_string())?;
    //         let path = file.path();
    //         if let Some(ext) = path.extension() {
    //             if ext == "vpk" {
    //
    //             }
    //         }
    //     }
    // }

    pub fn start_scan(&mut self) -> bool {
        if self.thread.is_none() {
            let mut queue = Arc::new(Mutex::new(VecDeque::new()));
            let addons = self.addons.clone();
            let thread = std::thread::spawn(move || scan_thread(queue.clone(), addons));
            self.thread = Some(thread);
        }
        true
    }
}

pub fn scan_thread(mut queue: Arc<Mutex<VecDeque<PathBuf>>>, addons: Arc<AddonStorageContainer>) {
    loop {
        let item = {
            let mut queue = queue.lock().unwrap();
            if queue.len() == 0 {
                info!("Queue is empty, thread is ending");
                break;
            }
            queue.pop_back().unwrap()
        };
        let addons = addons.clone();
        tauri::async_runtime::block_on(async move {
            scan_file(item, addons).await
        });
    }

}
pub async fn scan_file(path: PathBuf, addons: Arc<AddonStorageContainer>) -> Result<(), String> {
    let meta = path.metadata().unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();
    let addon_entry = {
        let addons = addons.lock().await;
        addons.get_by_filename(filename).await
            .map_err(|e| e.to_string())?
    };
    let mut should_update = false;
    if let Some(entry) = addon_entry {
        let last_modified = meta.modified().unwrap();
        // Check if file has been modified since scanned
        should_update = <DateTime<Utc>>::from(last_modified) > entry.updated_at;
        // if <DateTime<Utc>>::from(last_modified) > entry.updated_at {
        //     let addons = addons.lock().await;
        //     // TODO: Update entry data
        //     debug!("file has changed, updating entry {:?}", path);
        // }
        // return Ok(())
    }

    let (info, missions) = parse_addon(&path).await.map_err(|e| e.to_string())?;
    let mut addons = addons.lock().await;
    if let Some(title) = &info.title && let Some(version) = &info.version {
        if let Some(entry) = {
            addons.get_by_pk(title, version).await
                .map_err(|e| e.to_string())?
        } {
            // Addon's filename has changed but we found its corresponding entry. Update the filename
            addons.update_entry_pk(filename, version, title).await.unwrap();
        }
    }


    // File is either new or has been renamed, parse it and then we'll try to find it
    Ok(())
}

pub async fn parse_addon(path: &PathBuf) -> Result<(AddonInfo, Option<MissionInfo>), l4d2_addon_parser::Error> {
    let mut addon = L4D2Addon::from_path(&path)?;
    let info = addon.info()?
        .ok_or(l4d2_addon_parser::Error::VPKError("No addoninfo.txt".to_string()))?;
    let map = addon.missions()?;
    Ok((info, map))
}
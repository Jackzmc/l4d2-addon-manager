use l4d2_addon_parser::addon_list::AddonList;
use std::time::Duration;
use log::{debug, error, info};
use serde::Serialize;
use sqlx::__rt::spawn_blocking;
use tauri::State;
use crate::store::{AddonEntry, AddonFlags, AddonStorageContainer};
use crate::cfg::AppConfigContainer;
use crate::scan::ScannerContainer;

#[tauri::command]
pub async fn addons_counts(addons: State<'_, AddonStorageContainer>) -> Result<(u32, u32), String> {
    let addons = addons.lock().await;
    addons.counts().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn addons_list_managed(addons: State<'_, AddonStorageContainer>) -> Result<Vec<AddonEntry>, String> {
    let addons = addons.lock().await;
    addons.list().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn addons_list_workshop(addons: State<'_, AddonStorageContainer>) -> Result<Vec<AddonEntry>, String> {
    let addons = addons.lock().await;
    addons.list_workshop().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn addons_start_scan(cfg: State<'_, AppConfigContainer>, scanner: State<'_, ScannerContainer>) -> Result<(), String> {
    let addons_folder = {
        let cfg = cfg.lock().await;
        cfg.addons_folder.clone().ok_or_else(|| "no addon folder configured".to_string())?
    };
    let mut scanner = scanner.lock().unwrap();
    match scanner.start(addons_folder) {
        true => Ok(()),
        false => Err("A scan is already in progress".to_string())
    }
}

#[tauri::command]
pub async fn addons_abort_scan(scanner: State<'_, ScannerContainer>, reason: Option<String>) -> Result<(), String> {
    let mut scanner = scanner.lock().unwrap();
    scanner.abort(reason);
    Ok(())
}

#[derive(Serialize, Clone)]
pub enum ItemResult {
    Ok(String),
    Error(String, String)
}

#[tauri::command]
pub async fn addons_migrate(cfg: State<'_, AppConfigContainer>, ids: Vec<i64>) -> Result<Vec<ItemResult>, String> {
    let (workshop_folder, addons_folder, (steam, can_unsubscribe)) = {
        let cfg = cfg.lock().await;
        let addons_folder = cfg.addons_folder.as_ref().ok_or("addons folder missing".to_string())?.to_owned();
        let steam = cfg.steam();
        (addons_folder.join("workshop"), addons_folder, steam)
    };
    let mut i = 0;
    debug!("ws={:?} addons={:?} can_unsubscribe={}", workshop_folder, addons_folder, can_unsubscribe);
    // sync methods
    spawn_blocking(move || {
        let results: Vec<ItemResult> = ids.into_iter()
            .map(|id| {
                let filename = format!("{}.vpk", id);
                let src = workshop_folder.join(&filename);
                let dest = addons_folder.join(&filename);
                if let Err(e) = std::fs::copy(src, dest) {
                    return ItemResult::Error(filename, e.to_string());
                }
                if can_unsubscribe {
                    if let Err(e) = steam.unsubscribe(&id.to_string()) {
                        return ItemResult::Error(filename, e.to_string());
                    }
                    // Sleep in between requests so we don't hit steam api key
                    // with a ton (ids.len()) amount of requests at once
                    std::thread::sleep(Duration::from_millis(500 * i));
                }
                i += 1;
                ItemResult::Ok(filename)
            })
            .inspect(|result| {
                match result {
                    ItemResult::Ok(filename) => info!("Migrate {}: OK", filename),
                    ItemResult::Error(filename, err) => error!("Migrate {}: {}", filename, err)
                }
            })
            .collect();
        Ok(results)
    }).await
}

#[tauri::command]
pub async fn addons_unsubscribe(ids: Vec<i64>, cfg: State<'_, AppConfigContainer>, ) -> Result<Vec<ItemResult>, String> {
    let (steam, can_unsubscribe) = {
        let cfg = cfg.lock().await;
        cfg.steam()
    };
    if !can_unsubscribe {
        return Err("Can only unsubscribe if your own steam api key is provided".to_string());
    }
    spawn_blocking(move || {
        let mut i = 0;
        Ok(ids.into_iter().map(|id| {
            let id = id.to_string();
            if let Err(e) = steam.unsubscribe(&id) {
                return ItemResult::Error(id, e.to_string());
            }
            // Sleep in between requests so we don't hit steam api key
            // with a ton (ids.len()) amount of requests at once
            std::thread::sleep(Duration::from_millis(500 * i));
            i += 1;
            ItemResult::Ok(id)
        })
            .collect())
    }).await
}

#[tauri::command]
pub async fn addons_set_state(cfg: State<'_, AppConfigContainer>, filenames: Vec<String>, state: bool) -> Result<(), String> {
    // ASSUMPTION: Only running for addons in main folder, not workshop folder
    let addonslist_path = {
        let cfg = cfg.lock().await;
        cfg.addons_folder.as_ref().ok_or("addons folder missing".to_string())?.parent().unwrap().join("addonlist.txt")
    };
    debug!("addonlist.txt at {:?}", addonslist_path);
    // TODO: test disabling it via addonlist.txt (if it gets overwritten, works). if not then .disabled suffix
    let mut list = AddonList::new(&addonslist_path).map_err(|e| format!("failed to check state: {}", e))?;
    for filename in filenames {
        debug!("{} state = {}", filename, list.is_enabled(&filename));
        list.set_enabled(filename, state).map_err(|e| format!("failed to set state {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn addons_delete(cfg: State<'_, AppConfigContainer>, filenames: Vec<String>) -> Result<Vec<ItemResult>, String> {
    // ASSUMPTION: Only running for addons in main folder, not workshop folder
    let addons_folder = {
        let cfg = cfg.lock().await;
        cfg.addons_folder.as_ref().ok_or("addons folder missing".to_string())?.to_owned()
    };
    Ok(filenames.into_iter()
        .map(|filename| {
            let path = addons_folder.join(&filename);
            match trash::delete(&path){
                Ok(_) => ItemResult::Ok(filename),
                Err(e) => ItemResult::Error(filename, e.to_string())
            }
        })
        .collect()
    )
}
use std::ops::Deref;
use crate::modules::cfg::AppConfigContainer;
use crate::modules::store::{AddonEntry, AddonStorageContainer, FileHash, SelectedSort};
use crate::scan::{ScanSpeed, ScannerContainer};
use crate::util::get_addon_list;
use l4d2_addon_parser::addon_list::AddonList;
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use sqlx::__rt::spawn_blocking;
use std::time::Duration;
use tauri::{AppHandle, State};
use crate::modules::migrate::{migrate_workshop, unsubscribe_workshop};

#[tauri::command]
pub async fn addons_counts(addons: State<'_, AddonStorageContainer>) -> Result<(u32, u32), String> {
    let addons = addons.lock().await;
    addons.counts().await.map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn addons_list_managed(
    addons: State<'_, AddonStorageContainer>,
    cfg: State<'_, AppConfigContainer>,
    sort: Option<SelectedSort>
) -> Result<Vec<AddonEntry>, String> {
    let addon_list = get_addon_list(cfg).await;
    let addons = addons.lock().await;
    addons.list(addon_list, sort).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn addons_list_workshop(
    addons: State<'_, AddonStorageContainer>,
    cfg: State<'_, AppConfigContainer>,
    sort: Option<SelectedSort>
) -> Result<Vec<AddonEntry>, String> {
    let addon_list = get_addon_list(cfg).await;
    let addons = addons.lock().await;
    addons
        .list_workshop(addon_list, sort)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn addons_start_scan(
    cfg: State<'_, AppConfigContainer>,
    scanner: State<'_, ScannerContainer>,
    speed: Option<ScanSpeed>,
) -> Result<(), String> {
    let addons_folder = {
        let cfg = cfg.lock().await;
        cfg.addons_folder
            .clone()
            .ok_or_else(|| "no addon folder configured".to_string())?
    };
    let mut scanner = scanner.lock().await;
    match scanner.start(addons_folder, speed.unwrap_or_default()) {
        true => Ok(()),
        false => Err("A scan is already in progress".to_string()),
    }
}

#[tauri::command]
pub async fn addons_abort_scan(
    scanner: State<'_, ScannerContainer>,
    reason: Option<String>,
) -> Result<(), String> {
    let mut scanner = scanner.lock().await;
    scanner.abort(reason).await;
    Ok(())
}

#[derive(Serialize, Clone)]
#[serde(tag = "result")]
#[serde(rename_all = "lowercase")]
pub enum ItemResult {
    Ok { filename: String },
    Error { filename: String, error: String },
}
impl ItemResult {
    pub fn ok(filename: String) -> Self {
        ItemResult::Ok { filename }
    }
    pub fn error(filename: String, error: String) -> Self {
        ItemResult::Error { filename, error }
    }
}

#[tauri::command]
pub async fn addons_migrate(
    app: AppHandle,
    ids: Vec<i64>,
) -> Result<Vec<ItemResult>, String> {
    spawn_blocking(move || migrate_workshop(app, ids)).await
}

#[tauri::command]
pub async fn addons_unsubscribe(
    app: AppHandle,
    ids: Vec<i64>,
) -> Result<Vec<ItemResult>, String> {
    spawn_blocking(move || unsubscribe_workshop(app, ids)).await
}

#[tauri::command]
pub async fn addons_set_state(
    cfg: State<'_, AppConfigContainer>,
    filenames: Vec<String>,
    state: bool,
) -> Result<Vec<ItemResult>, String> {
    // ASSUMPTION: Only running for addons in main folder, not workshop folder
    let addonslist_path = {
        let cfg = cfg.lock().await;
        cfg.addons_folder
            .as_ref()
            .ok_or("addons folder missing".to_string())?
            .parent()
            .unwrap()
            .join("addonlist.txt")
    };
    // TODO: test disabling it via addonlist.txt (if it gets overwritten, works). if not then .disabled suffix
    let mut list =
        AddonList::new(&addonslist_path).map_err(|e| format!("failed to check state: {}", e))?;
    let results = filenames
        .into_iter()
        .map(
            |filename| match list.set_enabled(filename.to_string(), state) {
                Ok(()) => ItemResult::ok(filename),
                Err(err) => ItemResult::error(filename, err.to_string()),
            },
        )
        .collect();
    list.save()
        .map_err(|e| format!("failed to save addonlist.txt: {}", e))?;
    Ok(results)
}

#[tauri::command]
pub async fn addons_delete(
    cfg: State<'_, AppConfigContainer>,
    filenames: Vec<String>,
    addons: State<'_, AddonStorageContainer>,
) -> Result<Vec<ItemResult>, String> {
    // ASSUMPTION: Only running for addons in main folder, not workshop folder
    let addons_folder = {
        let cfg = cfg.lock().await;
        cfg.addons_folder
            .as_ref()
            .ok_or("addons folder missing".to_string())?
            .to_owned()
    };
    let results: Vec<ItemResult> = filenames
        .into_iter()
        .map(|filename| {
            let path = addons_folder.join(&filename);
            match trash::delete(&path) {
                Ok(_) => ItemResult::ok(filename),
                Err(e) => ItemResult::error(filename, e.to_string()),
            }
        })
        .collect();
    // Delete their entries from db
    let deleted_filenames: Vec<String> = results
        .iter()
        .filter_map(|result| match result {
            ItemResult::Ok { filename } => Some(filename.to_string()),
            _ => None,
        })
        .collect();
    let addons = addons.lock().await;
    addons
        .delete_filenames(deleted_filenames)
        .await
        .map_err(|e| e.to_string())?;
    Ok(results)
}

#[tauri::command]
pub async fn addons_tag_add(
    addons: State<'_, AddonStorageContainer>,
    id: String,
    tag: String
) -> Result<(), String> {
    let hash = FileHash::from_str(&id).map_err(|e| format!("bad id: {}", e))?;
    let addons = addons.lock().await;
    addons.add_tag(hash, tag).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn addons_tag_del(
    addons: State<'_, AddonStorageContainer>,
    id: String,
    tag: String
) -> Result<(), String> {
    let hash = FileHash::from_str(&id).map_err(|e| format!("bad id: {}", e))?;
    let addons = addons.lock().await;
    addons.del_tag(hash, tag).await.map_err(|e| e.to_string())
}
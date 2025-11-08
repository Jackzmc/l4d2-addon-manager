use tauri::State;
use crate::store::{AddonEntry, AddonFlags, AddonStorageContainer};
use crate::cfg::AppConfigContainer;
use crate::scan::ScannerContainer;

#[tauri::command]
pub async fn addons_list_managed(addons: State<'_, AddonStorageContainer>) -> Result<Vec<AddonEntry>, String> {
    let addons = addons.lock().await;
    addons.list(AddonFlags(0)).await.map_err(|e| e.to_string())
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

#[tauri::command]
pub async fn addons_migrate(ids: Vec<i64>) -> Result<(), String> {
    Err("not implemented".to_string())
}

#[tauri::command]
pub async fn addons_unsubscribe(ids: Vec<i64>) -> Result<(), String> {
    Err("not implemented".to_string())
}


#[tauri::command]
pub async fn addons_disable(filenames: Vec<String>) -> Result<(), String> {
    Err("not implemented".to_string())
}

#[tauri::command]
pub async fn addons_delete(filenames: Vec<String>) -> Result<(), String> {
    Err("not implemented".to_string())
}
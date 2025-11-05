use log::debug;
use tauri::State;
use crate::addons::AddonStorageContainer;
use crate::cfg::AppConfigContainer;

#[tauri::command]
pub async fn addons_list_managed() -> Result<Vec<()>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn addons_scan_managed(cfg: State<'_, AppConfigContainer>, addons: State<'_, AddonStorageContainer>) -> Result<(), String> {
    debug!("starting scan of addons");
    let addons_folder = {
        let cfg = cfg.lock().await;
        cfg.addons_folder.clone().ok_or_else(|| "no addon folder configured".to_string())?
    };
    let mut addons = addons.lock().await;
    addons.scan(addons_folder).await?;
    Ok(())
}

#[tauri::command]
pub async fn addons_list_workshop() -> Result<Vec<()>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn addons_scan_workshop() -> Result<(), String> {
    Ok(())
}

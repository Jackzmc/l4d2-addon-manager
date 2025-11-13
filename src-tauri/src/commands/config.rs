use crate::modules::cfg::{AppConfig, AppConfigContainer};
use crate::scan::{ScanSpeed, ScannerContainer};
use log::{debug, info};
use std::env::home_dir;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn choose_game_folder(app: tauri::AppHandle) -> Result<PathBuf, String> {
    debug!("opening dialog");

    let mut dialog = app
        .dialog()
        .file()
        .set_title("Choose Game Folder")
        .set_can_create_directories(false);
    if cfg!(windows) {
        dialog = dialog
            .set_directory("C:\\Program Files (x86)\\Steam\\steamapps\\common")
            .add_filter("left4dead2.exe", &["exe"])
            .set_file_name("left4dead2.exe");
    } else {
        let home_dir = home_dir().ok_or("could not acquire home dir".to_string())?;
        dialog = dialog
            .set_directory(
                home_dir
                    .join(".steam")
                    .join("steam")
                    .join("steamapps")
                    .join("common"),
            )
            .add_filter("left4dead2.exe", &["exe"])
            .set_file_name("left4dead2.exe");
    }
    let path = dialog
        .blocking_pick_file()
        .ok_or(String::from("failed to pick file"))?
        .into_path()
        .map_err(|e| e.to_string())?;
    let file_name = path.file_name().ok_or("invalid file".to_string())?;
    if file_name != "left4dead2.exe" && file_name != "left4dead2" {
        return Err(String::from("File must be a left4dead2 game executable"));
    }
    Ok(path.parent().unwrap().join("left4dead2").join("addons"))
}

#[tauri::command]
// Used by first time setup, trust its value
pub async fn set_game_folder(
    cfg: State<'_, AppConfigContainer>,
    path: String,
    scanner: State<'_, ScannerContainer>,
) -> Result<(), String> {
    debug!("setting addons folder to {}", path);
    let mut cfg = cfg.lock().await;
    let is_first_time = cfg.addons_folder.is_none();
    let path = PathBuf::from(path);
    cfg.addons_folder = Some(path.clone());
    // Start a scan at full speed if this is the first time
    if is_first_time {
        info!("First time setup, starting maximum scan");
        let mut scanner = scanner.lock().unwrap();
        scanner.start(path, ScanSpeed::Maximum);
    }
    cfg.save();
    Ok(())
}
#[tauri::command]
pub async fn set_config(
    app: AppHandle,
    cfg: State<'_, AppConfigContainer>,
    config: AppConfig,
) -> Result<(), String> {
    let mut cfg = cfg.lock().await;
    info!("set_config old {:?}", cfg);
    info!("set_config new {:?}", config);
    cfg.replace(config.clone())?;
    cfg.save();
    app.emit("config_changed", config).ok();
    Ok(())
}

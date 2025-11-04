use std::env::home_dir;
use std::path::PathBuf;
use tauri_plugin_dialog::{DialogExt, FilePath};
use log::debug;

#[tauri::command]
pub async fn choose_game_folder(app: tauri::AppHandle) -> Result<PathBuf, String> {
    debug!("opening dialog");

    let mut dialog = app.dialog()
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
            .set_directory(home_dir.join(".steam").join("steam").join("steamapps").join("common"))
            .add_filter("left4dead2.exe", &["exe"])
            .set_file_name("left4dead2.exe");
    }
    let path =  dialog.blocking_pick_file()
        .ok_or(String::from("failed to pick file"))?
        .into_path().map_err(|e| e.to_string())?;
    let file_name = path.file_name().ok_or("invalid file".to_string())?;
    if file_name != "left4dead2.exe" && file_name != "left4dead2" {
        return Err(String::from("File must be a left4dead2 game executable"));
    }
    Ok(path.join("left4dead2").join("addons"))
}

#[tauri::command]
pub async fn set_game_folder(app: tauri::AppHandle, path: String) {
    debug!("setting addons folder to {}", path);
}

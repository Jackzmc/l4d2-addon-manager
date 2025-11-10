use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::cfg::{AppConfig, StaticData};
use log::{debug, info};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use zip::result::{ZipError, ZipResult};
use zip::write::{FileOptions, SimpleFileOptions};
use zip::ZipWriter;
use crate::cfg::AppConfigContainer;
use crate::store::AddonStorageContainer;
use crate::util::SetRoute;

pub mod config;
pub mod addons;

#[derive(Serialize)]
pub struct InitData {
    initial_route: SetRoute,
    data: StaticData,
    config: AppConfig
}
#[tauri::command]
pub async fn init(config: State<'_, AppConfigContainer>, data: State<'_, StaticData>) -> Result<InitData, String> {
    let config = config.lock().await;
    let route_name = match config.addons_folder {
        Some(_) => {
            debug!("addon folder set, skipping setup");
            "addons-manual"
        },
        None => {
            debug!("addon folder not set, showing setup");
            "setup"
        }
    };
    let config = config.clone(); // copy the settings to send to UI
    debug!("init: initial route set to {}", route_name);
    Ok(InitData {
        initial_route: SetRoute {
            name: Some(route_name.to_string())
        },
        data: data.inner().clone(),
        config
    })
}

// TODO: move all this to export module, with proper multithreading for with_addons
#[tauri::command]
pub async fn export(app: AppHandle, data: State<'_, StaticData>, config: State<'_, AppConfigContainer>, with_addons: bool) -> Result<PathBuf, String> {
    let save_path = app
        .dialog()
        .file()
        .set_file_name("addon-manager-export.zip")
        .set_title("Choose Save Location")
        .add_filter("ZIP Archive", &["zip"])
        .blocking_save_file()
        .ok_or(String::from("failed to pick file"))?
        .into_path()
        .map_err(|e| e.to_string())?;
    let mut out_file = File::create(&save_path).unwrap();
    let data_dir = app.path().app_local_data_dir().unwrap();
    // let buff = std::io::Cursor::new(&mut out_file);
    let mut zip = zip::ZipWriter::new(&mut out_file);
    zip_file_path(&mut zip, "addon-manager.db", PathBuf::from(data_dir.join("addon-manager.db")));
    zip_file_path(&mut zip, "config.json", PathBuf::from(data_dir.join("config.json")));
    zip.start_file("version.txt", SimpleFileOptions::default()).unwrap();
    zip.write_all(data.app_version.as_bytes()).unwrap();
    if with_addons {
        let config = config.lock().await;
        zip_folder_path(&mut zip, "addons", config.addons_folder.as_ref().unwrap().clone(), false);
    }
    zip.finish().unwrap();
    info!("Export complete");
    Ok(save_path)
}

fn zip_file_path<T>(zip: &mut ZipWriter::<T>, file_name: &str, path: PathBuf) -> Result<(), ZipError> where T: std::io::Write + std::io::Seek {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    zip.start_file(file_name, SimpleFileOptions::default())?;
    file.read_to_end(&mut buffer)?;
    zip.write_all(&buffer)?;
    Ok(())
}

fn zip_folder_path<T>(zip: &mut ZipWriter::<T>, folder_name: &str, path: PathBuf, recursive: bool) -> Result<(), ZipError> where T: std::io::Write + std::io::Seek {
    zip.add_directory(folder_name, SimpleFileOptions::default())?;
    let dir = read_dir(path);
    let parent_path = Path::new(folder_name);
    for entry in dir.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = entry.file_name();
            let name = name.to_str().unwrap();
            let file_name = parent_path.join(name).to_string_lossy().to_string();
            debug!("file = {}; src = {:?}", file_name, path);
            zip_file_path(zip, &file_name, path).unwrap();
        } else if path.is_dir() {
            if recursive {
                unimplemented!("recursive folder");
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_database(addons: State<'_, AddonStorageContainer>, app: AppHandle) -> Result<(), String> {
    let addons = addons.lock().await;
    addons.danger_delete().await.map_err(|e| e.to_string())?;
    app.restart();
}

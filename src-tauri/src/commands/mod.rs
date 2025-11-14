use crate::modules::cfg::AppConfigContainer;
use crate::modules::cfg::{AppConfig, StaticData};
use crate::modules::export::export_app;
use crate::modules::store::AddonStorageContainer;
use crate::util::SetRoute;
use log::debug;
use serde::Serialize;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;
use steamlocate::{App, Error, Library};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

pub mod addons;
pub mod config;
pub mod logs;

#[derive(Serialize)]
pub struct InitData {
    initial_route: SetRoute,
    data: StaticData,
    config: AppConfig,
    addon_folder_suggestion: Option<PathBuf>,
}
#[tauri::command]
pub async fn init(
    app: AppHandle,
    config: State<'_, AppConfigContainer>,
    data: State<'_, StaticData>,
) -> Result<InitData, String> {
    let suggestion = {
        match steamlocate::SteamDir::locate().and_then(|steam_dir| steam_dir.find_app(550)) {
            Ok(Some((app, libr))) => { Some(libr.resolve_app_dir(&app).join("left4dead2/addons")) }
            _ => {
                match std::env::consts::OS {
                    "windows" => Some(PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Left 4 Dead2\left4dead2\addons")),
                    "linux" => {
                        app.path().home_dir()
                            .map(|home| home.join(".steam/steam/steamapps/common/Left 4 Dead 2/left4dead2/addons"))
                            .ok()
                    },
                    _ => None
                }
            }
        }
    };
    let config = config.lock().await;
    let route_name = match config.addons_folder {
        Some(_) => {
            debug!("addon folder set, skipping setup");
            "addons-manual"
        }
        None => {
            debug!("addon folder not set, showing setup");
            "setup"
        }
    };
    let config = config.clone(); // copy the settings to send to UI
    debug!("init: initial route set to {}", route_name);
    Ok(InitData {
        initial_route: SetRoute {
            name: Some(route_name.to_string()),
        },
        data: data.inner().clone(),
        addon_folder_suggestion: suggestion,
        config,
    })
}

// TODO: move all this to export module, with proper multithreading for with_addons
#[tauri::command]
pub async fn export(
    app: AppHandle,
    data: State<'_, StaticData>,
    config: State<'_, AppConfigContainer>,
    with_addons: bool,
) -> Result<PathBuf, String> {
    let app_version = data.app_version.clone();
    let addons_folder = if with_addons {
        let cfg = config.lock().await;
        cfg.addons_folder.as_ref().map(|p| p.clone())
    } else {
        None
    };
    let export_path = {
        let app = app.clone();
        tokio::task::spawn_blocking(move || export_app(app, app_version, addons_folder))
            .await
            .unwrap()?
    };
    app.opener()
        .open_path(export_path.to_string_lossy().to_string(), None::<&str>)
        .unwrap();
    Ok(export_path)
}
#[tauri::command]
pub async fn clear_database(
    addons: State<'_, AddonStorageContainer>,
    app: AppHandle,
) -> Result<(), String> {
    let addons = addons.lock().await;
    addons
        .danger_drop_database()
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}

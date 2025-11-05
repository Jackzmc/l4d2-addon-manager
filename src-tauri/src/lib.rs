use crate::cfg::AppConfig;
use crate::commands::config as cmd_config;
use crate::commands::addons as cmd_addons;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use log::debug;
use tauri::async_runtime::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;
use crate::addons::AddonStorage;

pub mod cfg;
mod commands;
pub mod util;
mod addons;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir().unwrap();

            let config = AppConfig::load(data_dir.join("config.json"));
            app.manage(Mutex::new(config));

            let db = AddonStorage::new(data_dir);
            app.manage(Mutex::new(db));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            cmd_config::choose_game_folder,
            cmd_config::set_game_folder,
            cmd_addons::addons_list_managed,
            cmd_addons::addons_scan_managed,
            cmd_addons::addons_list_workshop,
            cmd_addons::addons_scan_workshop
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

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

pub mod cfg;
mod commands;
pub mod util;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let cfg_path = app.path().app_config_dir().unwrap().join("config.json");
            let config = AppConfig::load(cfg_path);
            if config.addons_folder.is_some() {
                debug!("addon folder set, skipping setup");
                app.emit_str("set_route", "/app/addons/manual".to_string()).unwrap();
            } else {
                debug!("addon folder not set, showing setup");
                app.emit_str("set_route", "/".to_string()).unwrap();
            }

            app.manage(Mutex::new(config));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            cmd_config::choose_game_folder,
            cmd_config::set_game_folder,
            cmd_addons::addons_list_managed,
            cmd_addons::addons_list_workshop,

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use log::debug;
use crate::cfg::AppConfig;
use crate::commands::config as cmd_config;
use crate::commands::addons as cmd_addons;
use std::sync::{Arc};
use tauri::async_runtime::Mutex;
use tauri::{Manager, RunEvent};
use crate::store::{AddonStorage, AddonStorageContainer};
use crate::scan::AddonScanner;

pub mod cfg;
mod commands;
pub mod util;
mod store;
mod models;
mod scan;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_log::Builder::new()
            // Set default level to INFO, but our crate TRACE
            .level(log::LevelFilter::Info)
            .level_for("l4d2_addon_manager_lib", log::LevelFilter::Trace)
            .build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            app.manage(cfg::StaticData::new(app));
            let data_dir = app.path().app_local_data_dir().unwrap();

            let config = AppConfig::load(data_dir.join("config.json"));
            app.manage(Mutex::new(config));
            let db = tauri::async_runtime::block_on(async move {
                let db = AddonStorage::new(data_dir).await.expect("failed to create db");
                db.run_migrations().await.expect("migrations failed");
                let db = Arc::new(Mutex::new(db));
                db
            });
            app.manage(db.clone());

            let scanner = std::sync::Mutex::new(AddonScanner::new(db.clone(), app.handle().clone()));
            app.manage(scanner);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            cmd_config::choose_game_folder,
            cmd_config::set_game_folder,
            cmd_config::set_apikey,
            cmd_addons::addons_list_managed,
            cmd_addons::addons_list_workshop,
            cmd_addons::addons_start_scan,
            cmd_addons::addons_abort_scan,
            cmd_addons::addons_migrate,
            cmd_addons::addons_unsubscribe,
            cmd_addons::addons_disable,
            cmd_addons::addons_delete,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(|app, event| match event {
        RunEvent::ExitRequested {..} => {
            let db = app.state::<AddonStorageContainer>().inner().clone();
            // let db = db.blocking_lock();
            tauri::async_runtime::spawn(async move {
                let db = db.lock().await;
                debug!("cleaning up db...");
                db.close().await;
                debug!("cleaning up db... done");
            });
        },
        _ => {}
    })
}

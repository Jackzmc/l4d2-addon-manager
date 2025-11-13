use crate::cfg::AppConfig;
use crate::commands::addons as cmd_addons;
use crate::commands::config as cmd_config;
use crate::commands::logs as cmd_logs;
use crate::modules::cfg;
use crate::modules::store::{AddonStorage, AddonStorageContainer};
use crate::scan::AddonScanner;
use log::{LevelFilter, debug};
use std::str::FromStr;
use std::sync::Arc;
use tauri::async_runtime::Mutex;
use tauri::{Manager, RunEvent};
use tauri_plugin_log::Target;
use tauri_plugin_log::TargetKind;

mod commands;
mod models;
mod modules;
mod scan;
pub mod util;

fn log_level() -> LevelFilter {
    let level = LevelFilter::from_str(option_env!("APP_LOG_LEVEL").unwrap_or("trace"))
        .expect("invalid log level");
    println!("log level: {}", level);
    level
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(
            tauri_plugin_log::Builder::new()
                // Set default level to INFO, but our crate TRACE
                .level(log::LevelFilter::Info)
                .level_for("l4d2_addon_manager_lib", log_level())
                // in addition to defaults, also send to frontend
                .target(Target::new(TargetKind::Webview))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(cfg::StaticData::new(app));
            let data_dir = app.path().app_local_data_dir().unwrap();

            let config = AppConfig::load(data_dir.join("config.json"));
            app.manage(Mutex::new(config));
            let db = tauri::async_runtime::block_on(async move {
                let db = AddonStorage::new(data_dir)
                    .await
                    .expect("failed to create db");
                db.run_migrations().await.expect("migrations failed");
                let db = Arc::new(Mutex::new(db));
                db
            });
            app.manage(db.clone());

            let scanner =
                std::sync::Mutex::new(AddonScanner::new(db.clone(), app.handle().clone()));
            app.manage(scanner);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            commands::export,
            commands::clear_database,
            cmd_logs::get_logs,
            cmd_logs::open_logs_folder,
            cmd_logs::upload_logs,
            cmd_config::choose_game_folder,
            cmd_config::set_game_folder,
            cmd_config::set_config,
            cmd_addons::addons_counts,
            cmd_addons::addons_list_managed,
            cmd_addons::addons_list_workshop,
            cmd_addons::addons_start_scan,
            cmd_addons::addons_abort_scan,
            cmd_addons::addons_migrate,
            cmd_addons::addons_unsubscribe,
            cmd_addons::addons_set_state,
            cmd_addons::addons_delete,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(|app, event| match event {
        RunEvent::ExitRequested { .. } => {
            let db = app.state::<AddonStorageContainer>().inner().clone();
            // let db = db.blocking_lock();
            tauri::async_runtime::spawn(async move {
                let db = db.lock().await;
                debug!("cleaning up db...");
                db.close().await;
                debug!("cleaning up db... done");
            });
        }
        _ => {}
    })
}

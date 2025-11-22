use std::path::PathBuf;
use std::time::Duration;
use log::{debug, error, info, trace};
use steam_workshop_api::SteamWorkshop;
use tauri::{AppHandle, Manager, State};
use crate::commands::addons::ItemResult;
use crate::modules::cfg::{AppConfig, AppConfigContainer};

pub fn migrate_workshop(app: AppHandle, ids: Vec<i64>) -> Result<Vec<ItemResult>, String> {
    let cfg = app.state::<AppConfigContainer>();
    let cfg = cfg.blocking_lock();
    let addons_folder = cfg
        .addons_folder
        .as_ref()
        .ok_or("addons folder missing".to_string())?
        .to_owned();
    let workshop_folder = addons_folder.join("workshop");
    let (steam, can_unsubscribe) = cfg.steam();
    drop(cfg);
    let mut i = 0;
    debug!(
        "ws={:?} addons={:?} can_unsubscribe={} ids={:?}",
        workshop_folder, addons_folder, can_unsubscribe, ids
    );

    let results = ids
        .into_iter()
        .map(|id| {
            let filename = format!("{}.vpk", id);
            let src = workshop_folder.join(&filename);
            let dest = addons_folder.join(&filename);
            trace!("cp {:?} -> {:?}", src, dest);
            if let Err(e) = std::fs::copy(src, dest) {
                return ItemResult::error(filename, e.to_string());
            }
            if can_unsubscribe {
                trace!("performing unsubscribe");
                if let Err(e) = steam.unsubscribe(&id.to_string()) {
                    return ItemResult::error(filename, e.to_string());
                }
                // Sleep in between requests so we don't hit steam api key
                // with a ton (ids.len()) amount of requests at once
                trace!("sleep {}ms", 500*i);
                std::thread::sleep(Duration::from_millis(500 * i));
            }
            i += 1;
            ItemResult::ok(filename)
        })
        .inspect(|result| match result {
            ItemResult::Ok { filename } => info!("Migrate {}: OK", filename),
            ItemResult::Error { filename, error } => error!("Migrate {}: {}", filename, error),
        })
        .collect();
    Ok(results)
}

pub fn unsubscribe_workshop(app: AppHandle, ids: Vec<i64>) -> Result<Vec<ItemResult>, String> {
    let cfg = app.state::<AppConfigContainer>();
    let (steam, can_unsubscribe) = {
        let cfg = cfg.blocking_lock();
        cfg.steam()
    };
    if !can_unsubscribe {
        return Err("Can only unsubscribe if your own steam api key is provided".to_string());
    }
    let mut i = 0;
    Ok(ids
        .into_iter()
        .map(|id| {
            let id = id.to_string();
            if let Err(e) = steam.unsubscribe(&id) {
                return ItemResult::error(id, e.to_string());
            }
            // Sleep in between requests so we don't hit steam api key
            // with a ton (ids.len()) amount of requests at once
            std::thread::sleep(Duration::from_millis(500 * i));
            i += 1;
            ItemResult::ok(id)
        })
        .collect())
}

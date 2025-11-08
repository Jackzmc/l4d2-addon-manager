use crate::cfg::StaticData;
use log::debug;
use serde::Serialize;
use tauri::{AppHandle, State};
use crate::cfg::AppConfigContainer;
use crate::scan::ScannerContainer;
use crate::util::SetRoute;

pub mod config;
pub mod addons;

#[derive(Serialize)]
pub struct InitData {
    initial_route: SetRoute,
    data: StaticData,
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
    debug!("init: initial route set to {}", route_name);
    Ok(InitData {
        initial_route: SetRoute {
            name: Some(route_name.to_string())
        },
        data: data.inner().clone(),
    })
}
use l4d2_addon_parser::addon_list::AddonList;
use log::error;
use serde::Serialize;
use tauri::State;
use crate::cfg::AppConfigContainer;

#[derive(Debug, Serialize)]
pub struct SetRoute {
    pub name: Option<String>,
}

pub async fn get_addon_list(cfg: State<'_, AppConfigContainer>) -> Option<AddonList> {
    let cfg = cfg.lock().await;
    cfg.addons_folder.as_ref()
        .and_then(|folder| match AddonList::new(&folder.parent().unwrap().join("addonlist.txt")) {
            Ok(list) => Some(list),
            Err(e) => {
                error!("Failed to load addonlist.txt: {}", e);
                None
            }
        })
}
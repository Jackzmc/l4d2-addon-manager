use std::fmt::{Display, Formatter};
use std::fs::Metadata;
use l4d2_addon_parser::addon_list::AddonList;
use log::{error, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use crate::modules::cfg::AppConfigContainer;

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
                warn!("loading addonlist.txt: {}", e);
                None
            }
        })
}

#[derive(Debug, Serialize, Clone)]
pub enum NotificationType {
    Info,
    Error,
    Warn,
    Custom(String)
}
impl Display for NotificationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Info => write!(f, "info"),
            NotificationType::Error => write!(f, "error"),
            NotificationType::Warn => write!(f, "warn"),
            NotificationType::Custom(s) => write!(f, "{}", s)
        }
    }
}
#[derive(Debug, Serialize, Clone)]
pub struct Notification {
    #[serde(rename = "type")]
    pub _type: NotificationType,
    pub title: String,
    pub text: Option<String>
}
impl Notification {
    pub fn new(typ: NotificationType, title: String, text: Option<String>) -> Self {
        Self { _type: typ, title, text }
    }

    pub fn send(self, app: AppHandle) {
        app.emit("notify", self).expect("failed to send notification");
    }
}

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

pub fn get_file_size(meta: &Metadata) -> i64 {
    #[cfg(unix)]
    return meta.size() as i64;
    #[cfg(windows)]
    return meta.file_size() as i64;
}
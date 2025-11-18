// Can guarantee id is 4 digits at minimum.
// IDs are sequential, L4D2 Workshop came out after the 10000th addon was released
use crate::modules::store::AddonFlags;
use l4d2_addon_parser::{AddonContent, AddonInfo};
use log::{info, warn};
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

static WORKSHOP_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://steamcommunity.com/sharedfiles/filedetails/\?id=(\d+)").unwrap()
});
static WORKSHOP_FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{4,}").unwrap());

/// Performs a scan of directory returning list of pathbufs
pub(super) fn get_vpks_in_dir(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    info!("Scanning addons at {}", path.display());
    let dir = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for file in dir {
        let file = file.map_err(|e| e.to_string())?;
        let path = file.path();
        if let Some(ext) = path.extension() {
            if ext == "vpk" {
                list.push(path);
            }
        }
    }
    Ok(list)
}

/// Attempts to extract workshop ID from addon url or filename
pub(super) fn find_workshop_id(filename: &str, addon: &AddonInfo) -> Option<i64> {
    // We try filename first, so the user can overwrite with whatever id and change it easily
    // Then fall back to the addon's set url, if set. That way if they have their own custom version,
    // they can still change it.

    // Try to get it from filename
    if let Some(cap) = WORKSHOP_FILE_REGEX.find(filename) {
        let id = cap.as_str().parse::<i64>().expect("regex \\d failed to parse");
        return Some(id);
    }

    //Try to get from addon's info url
    if let Some(url) = &addon.addon_url {
        if let Some(capture) = WORKSHOP_URL_REGEX.captures(url) {
            let id = capture.get(1).expect("capture group 1 missing").as_str();
            return Some(id.parse::<i64>().expect("regex \\d failed to parse"));
        }
    }

    None
}

impl Into<AddonFlags> for &AddonContent {
    fn into(self) -> AddonFlags {
        let mut flags = AddonFlags::empty();
        if self.is_map {
            flags |= AddonFlags::CAMPAIGN;
        }
        if self.is_survivor {
            flags |= AddonFlags::SURVIVOR;
        }
        if self.is_script {
            flags |= AddonFlags::SCRIPT;
        }
        if self.is_weapon {
            flags |= AddonFlags::WEAPON;
        }
        if self.is_sound || self.is_music {
            flags |= AddonFlags::SOUND;
        }
        flags
    }
}

pub(super) fn get_workshop_folder_ws_ids(path: &PathBuf) -> Vec<i64> {
    match get_vpks_in_dir(&path.join("workshop")) {
        Ok(list) => list
            .into_iter()
            .map(|item| item.file_stem().unwrap().to_string_lossy().parse::<i64>())
            // Remove any files that don't have a valid ID:
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect(),
        Err(e) => {
            warn!("failed to scan workshop dir: {}", e);
            Vec::new()
        }
    }
}

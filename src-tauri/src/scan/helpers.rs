// Can guarantee id is 4 digits at minimum.
// IDs are sequential, L4D2 Workshop came out after the 10000th addon was released
use l4d2_addon_parser::AddonInfo;
use crate::store::AddonFlags;
use std::path::PathBuf;
use regex::Regex;
use std::sync::LazyLock;
use log::info;

static WORKSHOP_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https://steamcommunity.com/sharedfiles/filedetails/\?id=(\d+)").unwrap());
static WORKSHOP_FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{4,}").unwrap());

/// Performs a scan of directory returning list of pathbufs
pub(super) fn get_vpks_in_dir(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    info!("Scanning addons at {}", path.display());
    let dir = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?;
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
pub(super) fn find_workshop_id(path: &PathBuf, addon: &AddonInfo) -> Option<i64> {
    // Try URL first, as we can guarantee from there
    if let Some(url) = &addon.addon_url {
        if let Some(capture) = WORKSHOP_URL_REGEX.captures(url) {
            let id = capture.get(1).unwrap().as_str();
            return Some(id.parse::<i64>().unwrap());
        }
    }

    // Try to get it from filename
    let filename = path.file_name().unwrap().to_str().unwrap();
    if let Some(cap) = WORKSHOP_FILE_REGEX.find(filename) {
        let id = cap.as_str().parse::<i64>().unwrap();
        return Some(id);
    }
    None
}

pub(super) fn get_addon_flags(info: &AddonInfo) -> AddonFlags {
    let mut flags = AddonFlags::empty();
    if info.content.is_map {
        flags |= AddonFlags::CAMPAIGN;
    }
    if info.content.is_survivor {
        flags |= AddonFlags::SURVIVOR;
    }
    if info.content.is_script {
        flags |= AddonFlags::SCRIPT;
    }
    if info.content.is_weapon {
        flags |= AddonFlags::WEAPON;
    }
    flags
}
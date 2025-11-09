use std::fs;
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use steam_workshop_api::SteamWorkshop;
use tauri::async_runtime::Mutex;

#[derive(Serialize, Clone)]
pub struct StaticData {
    pub app_version: String,
    pub git_commit: Option<String>
}
impl StaticData {
    pub fn new(app: &tauri::App) -> Self {
        Self {
            app_version: app.package_info().version.to_string(),
            git_commit: option_env!("GIT_COMMIT").map(|s| s.to_string()),
        }
    }
}

pub type AppConfigContainer = Mutex<AppConfig>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    #[serde(skip)]
    _save_path: PathBuf,

    pub addons_folder: Option<PathBuf>,
    pub steam_apikey: Option<String>
}

impl AppConfig {
    pub fn load(path_buf: PathBuf) -> Self {
        let mut config: AppConfig = match std::fs::File::open(&path_buf) {
            Ok(file) => {
                debug!("loaded config from {:?}", path_buf);
                serde_json::from_reader(file).expect("failed to read config file")
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    panic!("{}", e);
                }
                debug!("no config file found, loading default config");
                Default::default()
            }
        };
        config._save_path = path_buf;
        config
    }

    pub fn save(&self) {
        debug!("Saving config to {:?}", self._save_path);
        fs::create_dir_all(self._save_path.parent().unwrap()).ok();
        serde_json::to_writer(std::fs::File::create(&self._save_path).unwrap(), &self).unwrap();
    }

    /// Get an instance of SteamWorkshop client, with user's apikey if they set it
    /// returns true if apikey set
    pub fn steam(&self) -> (SteamWorkshop, bool) {
        let mut steam = SteamWorkshop::new();
        if let Some(apikey) = &self.steam_apikey {
            steam.set_apikey(Some(apikey.to_owned()));
            return (steam, true)
        }
        (steam, false)
    }
}

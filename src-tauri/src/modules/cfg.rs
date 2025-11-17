use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use serde_with::serde_as;
use steam_workshop_api::SteamWorkshop;
use tauri::async_runtime::Mutex;

#[derive(Serialize, Clone)]
pub struct StaticData {
    pub app_version: String,
    pub git_commit: Option<String>,
    pub is_prod: bool
}
impl StaticData {
    pub fn new(app: &tauri::App) -> Self {
        Self {
            app_version: app.package_info().version.to_string(),
            git_commit: option_env!("GIT_COMMIT").map(|s| s.to_string()),
            is_prod: !cfg!(debug_assertions)
        }
    }
}

pub type AppConfigContainer = Mutex<AppConfig>;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde_as]
pub struct AppConfig {
    #[serde(skip)]
    _save_path: PathBuf,

    pub addons_folder: Option<PathBuf>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub steam_apikey: Option<String>,

    #[serde(default = "default_as_true")]
    pub startup_scan: bool,
    #[serde(default)]
    pub startup_telemetry: bool
}
fn default_as_true() -> bool {
    true
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
            return (steam, true);
        }
        (steam, false)
    }

    pub fn validate(&self, new_config: &Self) -> Result<(), String> {
        if let Some(key) = &new_config.steam_apikey {
            if key.len() > 0 && key.len() != 32 {
                return Err("Steam API Key must be 32 characters long".to_string());
            }
        }
        if let Some(addons_folder) = &new_config.addons_folder {
            let meta = fs::metadata(addons_folder.as_path())
                .map_err(|_| "Addons folder must exist and be readable".to_string())?;
            if !meta.is_dir() {
                return Err("Addons folder must be a directory".to_string());
            }
        }

        Ok(())
    }
    /// Tries to replace config with a new config, after the new settings are validated
    pub fn replace(&mut self, new_config: Self) -> Result<(), String> {
        self.validate(&new_config)?;
        self.steam_apikey = new_config.steam_apikey;
        self.addons_folder = new_config.addons_folder;
        self.startup_scan = new_config.startup_scan;
        self.startup_telemetry = new_config.startup_telemetry;
        Ok(())
    }
}

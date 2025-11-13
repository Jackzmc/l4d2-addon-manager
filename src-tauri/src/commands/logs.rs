use crate::util::{Notification, NotificationType};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, Read};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize)]
pub struct LogEntry {
    message: String,
}
#[tauri::command]
pub async fn get_logs(app: AppHandle) -> Result<Vec<LogEntry>, String> {
    let logs_path = app
        .path()
        .app_local_data_dir()
        .unwrap()
        .join("logs")
        .join(format!("{}.log", env!("CARGO_PKG_NAME")));
    let file = File::open(logs_path).map_err(|e| e.to_string())?;
    let buff = std::io::BufReader::new(file);
    Ok(buff
        .lines()
        .map(|l| LogEntry {
            message: l.unwrap(),
        })
        .collect())
}

#[tauri::command]
pub async fn open_logs_folder(app: AppHandle) -> Result<(), String> {
    let logs_path = app.path().app_local_data_dir().unwrap().join("logs");
    debug!("logs_path = {:?}", logs_path);
    app.opener()
        .open_path(logs_path.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

static UPLOAD_LOGS_EXPIRES: Duration = Duration::from_secs(60 * 60 * 24 * 4); // 4d
#[derive(Deserialize)]
struct UploadedLogResult {
    name: String,
    url: String,
    expires: u64,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "deleteToken")]
    delete_token: String,
}

#[tauri::command]
pub async fn upload_logs(app: AppHandle) -> Result<(), String> {
    let logs_path = app
        .path()
        .app_local_data_dir()
        .unwrap()
        .join("logs")
        .join(format!("{}.log", env!("CARGO_PKG_NAME")));
    let client = reqwest::Client::new();
    let mut content = {
        let mut file = File::open(logs_path).map_err(|e| e.to_string())?;
        let mut string = String::new();
        file.read_to_string(&mut string)
            .map_err(|e| e.to_string())?;
        string
    };
    // add debug info
    let prefix = format!(
        r#"----------------------------------------------------
version: {} {}
OS: (todo)
time: {}
----------------------------------------------------
"#,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    content.insert_str(0, &prefix);

    debug!("uploading logs.... ({})", content.len());
    Notification::new(
        NotificationType::Info,
        "Uploading logs...".to_string(),
        None,
    )
    .send(&app);
    let result = client
        .post(format!(
            "https://paste.jackz.me/paste?expires={}",
            UPLOAD_LOGS_EXPIRES.as_secs()
        ))
        .header("Content-Type", "text/plain")
        .header(
            "User-Agent",
            format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )
        .body(content)
        .send()
        .await
        .map_err(|e| format!("failed to upload: {}", e))?
        .json::<UploadedLogResult>()
        .await
        .map_err(|e| format!("failed to parse response: {}", e))?;
    debug!(
        "log id={} url={} delete_token={}",
        result.name, result.url, result.delete_token
    );
    Notification::new(
        NotificationType::Info,
        "Logs uploaded".to_string(),
        Some(format!("URL: {}", result.url)),
    )
    .send(&app);
    app.opener()
        .open_path(result.url, None::<&str>)
        .map_err(|e| e.to_string())
}

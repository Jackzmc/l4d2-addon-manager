use log::{debug, info};
use std::fs::{File, read_dir};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::{App, AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};
use crate::util::defs::ProgressPayload;

/// Export the app. If addons_folder set, addons are included
pub fn export_app(
    app: AppHandle,
    app_version: String,
    addons_folder: Option<PathBuf>,
) -> Result<PathBuf, String> {
    info!(
        "Starting app export (with_addons={})",
        addons_folder.is_some()
    );
    let now = Instant::now();
    let save_path = prompt_save_location(&app)?;
    let mut file = File::create(&save_path).map_err(|e| format!("create file: {}", e))?;
    let mut zip = zip::ZipWriter::new(&mut file);

    let data_dir = app
        .path()
        .app_local_data_dir()
        .expect("could not find data dir");
    zip_file_path(
        &mut zip,
        "addon-manager.db",
        PathBuf::from(data_dir.join("addon-manager.db")),
        SimpleFileOptions::default(),
    )
    .map_err(|e| format!("zipping db: {}", e))?;
    zip_file_path(
        &mut zip,
        "config.json",
        PathBuf::from(data_dir.join("config.json")),
        SimpleFileOptions::default(),
    )
    .map_err(|e| format!("zipping config: {}", e))?;
    zip_file_content(
        &mut zip,
        "version.txt",
        app_version.as_bytes(),
        SimpleFileOptions::default(),
    )
    .map_err(|e| format!("zipping version: {}", e))?;

    if let Some(addons_folder) = addons_folder {
        zip.add_directory("addons", SimpleFileOptions::default()).unwrap();
        let files = export_get_addon_files("addons", addons_folder, false).unwrap();
        let mut progress = ProgressPayload::new(0, files.len() as u32);
        for file in files.into_iter() {
            let (file_name, path) = file;
            zip_file_path(&mut zip, &file_name, path, SimpleFileOptions::default().compression_method(CompressionMethod::Stored)).unwrap();
            app.emit("export_progress", progress.clone()).unwrap();
            progress.value += 1;
        }
    }

    info!(
        "App export complete. Time elapsed: {}",
        now.elapsed().as_secs()
    );
    Ok(save_path)
}

fn prompt_save_location(app: &AppHandle) -> Result<PathBuf, String> {
    app.dialog()
        .file()
        .set_file_name("addon-manager-export.zip")
        .set_title("Choose Save Location")
        .add_filter("ZIP Archive", &["zip"])
        .blocking_save_file()
        .ok_or(String::from("failed to pick file"))?
        .into_path()
        .map_err(|e| e.to_string())
}

fn zip_file_path<T>(
    zip: &mut ZipWriter<T>,
    file_name: &str,
    path: PathBuf,
    options: SimpleFileOptions,
) -> Result<(), ZipError>
where
    T: std::io::Write + std::io::Seek,
{
    let mut file = File::open(path)?;
    zip.start_file(file_name, options)?;
    let mut buffer = vec![0; 32_000];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        zip.write_all(&buffer[..n])?;
    }
    Ok(())
}

fn zip_file_content<T>(
    zip: &mut ZipWriter<T>,
    file_name: &str,
    content: &[u8],
    options: SimpleFileOptions,
) -> Result<(), ZipError>
where
    T: std::io::Write + std::io::Seek,
{
    zip.start_file(file_name, options)?;
    zip.write_all(content)?;
    Ok(())
}

fn zip_folder_path<T>(
    zip: &mut ZipWriter<T>,
    folder_name: &str,
    path: PathBuf,
    recursive: bool,
) -> Result<(), ZipError>
where
    T: std::io::Write + std::io::Seek,
{
    zip.add_directory(folder_name, SimpleFileOptions::default())?;
    let dir = read_dir(path);
    let parent_path = Path::new(folder_name);
            for entry in dir? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                let file_name = parent_path.join(name).to_string_lossy().to_string();
                debug!("file = {}; src = {:?}", file_name, path);
                let options =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
                // .large_file();
                zip_file_path(zip, &file_name, path, options)?;
            } else if path.is_dir() {
                if recursive {
                    unimplemented!("recursive folder");
                }
            }
        }
    }
    Ok(())
}

fn export_get_addon_files(
    folder_name: &str,
    path: PathBuf,
    recursive: bool,
) -> Result<Vec<(String, PathBuf)>, std::io::Error>
{
    let dir = read_dir(path)?;
    let parent_path = Path::new(folder_name);
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for entry in dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                let file_name = parent_path.join(name).to_string_lossy().to_string();
                files.push((file_name, path));
            } else if path.is_dir() {
                if recursive {
                    unimplemented!("recursive folder");
                }
            }
        }
    }
    Ok(files)
}


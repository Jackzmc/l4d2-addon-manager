use crate::commands::config;

mod commands;

#[tauri::command]
fn get_managed_addons() -> Vec<()> {
    vec![]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            config::choose_game_folder, config::set_game_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

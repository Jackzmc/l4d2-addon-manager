#[tauri::command]
pub async fn addons_list_managed() -> Result<Vec<()>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn addons_list_workshop() -> Result<Vec<()>, String> {
    Ok(vec![])
}
#[tauri::command]
pub fn set_provider_key(key_ref: String, secret: String) -> Result<(), String> {
    voxflow_secrets::set_secret(&key_ref, &secret).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_provider_key(key_ref: String) -> Result<(), String> {
    voxflow_secrets::delete_secret(&key_ref).map_err(|e| e.to_string())
}

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub fn get_config(app: AppHandle, key: String) -> Result<Option<String>, String> {
    Ok(Some(get_config_inner(&app, &key)))
}

pub fn get_config_inner(app: &AppHandle, key: &str) -> String {
    let store = match app.store("speek2words.json") {
        Ok(s) => s,
        Err(_) => return String::new(),
    };

    match store.get(key) {
        Some(v) => v.as_str().unwrap_or("").to_string(),
        None => String::new(),
    }
}

#[tauri::command]
pub fn set_config(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let store = app
        .store("speek2words.json")
        .map_err(|e| format!("Store get: {}", e))?;

    store.set(key, serde_json::Value::String(value));
    store.save().map_err(|e| format!("Store save: {}", e))?;

    Ok(())
}

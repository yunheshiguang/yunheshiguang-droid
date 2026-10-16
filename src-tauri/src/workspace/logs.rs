use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

use super::blocking::run_blocking;

#[tauri::command]
pub async fn reveal_log_directory(app: tauri::AppHandle) -> Result<(), String> {
    let directory = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;

    run_blocking(move || {
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        app.opener()
            .open_path(directory.to_string_lossy(), None::<&str>)
            .map_err(|error| error.to_string())
    })
    .await
}

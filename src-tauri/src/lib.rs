mod parser;

use tauri::Manager;

#[tauri::command]
async fn get_prompts(app: tauri::AppHandle, refresh: bool) -> Result<parser::ApiResponse, String> {
    let cache_dir = app.path().app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("github-markdown-cache");
        
    parser::load_and_parse_prompts(cache_dir, refresh).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .setup(|app| {
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![get_prompts])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

mod parser;

use tauri::Manager;
use std::path::PathBuf;

fn prompt_cache_dir(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(value) = std::env::var("PROMPT_LIBRARY_CACHE_DIR") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }

    // 便携版优先读取 EXE 同目录的预热缓存。
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join(".cache").join("github-markdown");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // 开发模式优先复用 Node 版已经生成的项目缓存。
    if let Ok(current_dir) = std::env::current_dir() {
        let mut candidates = vec![current_dir.join(".cache").join("github-markdown")];
        if let Some(parent) = current_dir.parent() {
            candidates.push(parent.join(".cache").join("github-markdown"));
        }
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
    }

    app.path()
        .app_cache_dir()
        .map(|dir| dir.join("github-markdown"))
        .unwrap_or_else(|_| std::env::temp_dir().join("prompt-library-viewer").join("github-markdown"))
}

#[tauri::command]
async fn get_prompts(app: tauri::AppHandle, refresh: bool) -> Result<parser::ApiResponse, String> {
    let cache_dir = prompt_cache_dir(&app);

    parser::load_and_parse_prompts(cache_dir, refresh).await
}

#[tauri::command]
fn start_dragging(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    if let Ok(true) = window.is_maximized() {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .setup(|_app| {
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        get_prompts,
        start_dragging,
        minimize_window,
        maximize_window,
        close_window
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

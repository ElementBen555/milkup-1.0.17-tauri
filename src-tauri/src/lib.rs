//! Milkup 后端 — Tauri 命令注册 + 文件监视 + 拖放 + 启动文件 + 单实例

mod commands;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

// ─── 全局状态 ──────────────────────────────────────────────

struct WatchState {
    file_watcher: Option<notify::RecommendedWatcher>,
    watched_files: HashSet<String>,
    dir_watcher: Option<notify::RecommendedWatcher>,
    last_dir_emit: std::time::Instant,
}

// ─── 文件监视 ──────────────────────────────────────────────

#[tauri::command]
fn watch_files(state: tauri::State<Mutex<WatchState>>, app: tauri::AppHandle, file_paths: Vec<String>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    // 初始化 watcher（若尚未创建）
    if guard.file_watcher.is_none() {
        let app_clone = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    for path in &event.paths {
                        let _ = app_clone.emit("file-changed", path.to_string_lossy().to_string());
                    }
                }
            }
        }).map_err(|e| format!("创建文件监视器失败: {e}"))?;

        guard.file_watcher = Some(watcher);
    }

    // 计算需要添加和移除的文件（先收集，后操作）
    let current: HashSet<String> = file_paths.iter().cloned().collect();
    let to_add: Vec<String> = file_paths.iter()
        .filter(|p| !guard.watched_files.contains(p.as_str()))
        .cloned()
        .collect();
    let to_remove: Vec<String> = guard.watched_files.iter()
        .filter(|p| !current.contains(p.as_str()))
        .cloned()
        .collect();

    {
        if let Some(ref mut watcher) = guard.file_watcher {
            for path in &to_add {
                if Path::new(path).exists() {
                    watcher.watch(Path::new(path), RecursiveMode::NonRecursive)
                        .map_err(|e| format!("监视文件失败 {path}: {e}"))?;
                }
            }
            for path in &to_remove {
                let _ = watcher.unwatch(Path::new(path));
            }
        }
    }

    // 更新跟踪集合
    for path in to_add {
        guard.watched_files.insert(path);
    }
    for path in to_remove {
        guard.watched_files.remove(&path);
    }

    Ok(())
}

#[tauri::command]
fn watch_directory(state: tauri::State<Mutex<WatchState>>, app: tauri::AppHandle, dir_path: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.dir_watcher = None;
    drop(guard);

    let app_clone = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)) {
                if let Ok(mut g) = app_clone.state::<Mutex<WatchState>>().lock() {
                    let now = std::time::Instant::now();
                    if now.duration_since(g.last_dir_emit).as_millis() > 300 {
                        g.last_dir_emit = now;
                        drop(g);
                        let _ = app_clone.emit("workspace-directory-changed", ());
                    }
                }
            }
        }
    }).map_err(|e| format!("创建目录监视器失败: {e}"))?;

    watcher
        .watch(Path::new(&dir_path), RecursiveMode::NonRecursive)
        .map_err(|e| format!("监视目录失败: {e}"))?;

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.dir_watcher = Some(watcher);
    Ok(())
}

#[tauri::command]
fn unwatch_directory(state: tauri::State<Mutex<WatchState>>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    if let Some(w) = guard.dir_watcher.take() {
        drop(w);
    }
    Ok(())
}

#[tauri::command]
fn quit_and_install() -> Result<(), String> {
    std::process::exit(0);
}

// ─── 启动入口 ──────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ─── 单实例锁 (Unix Socket) ───────────────────────────
    let socket_path = Path::new("/tmp/milkup.sock");

    // 尝试连接到已运行的实例
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let payload = args.join("\n");
        let _ = stream.write_all(payload.as_bytes());
        std::process::exit(0);
    }

    // 没有运行实例，创建监听
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).expect("创建 Unix Socket 失败");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Mutex::new(WatchState {
            file_watcher: None,
            watched_files: HashSet::new(),
            dir_watcher: None,
            last_dir_emit: std::time::Instant::now(),
        }))
        .invoke_handler(tauri::generate_handler![
            commands::file::open_file,
            commands::file::save_file,
            commands::file::save_file_as,
            commands::file::read_file_by_path,
            commands::file::is_read_only,
            commands::file::cleanup_local_images,
            commands::window::window_control,
            commands::window::close_discard,
            commands::window::get_window_bounds,
            commands::window::set_title,
            commands::window::open_theme_editor,
            commands::window::tear_off_tab,
            commands::window::merge_tab,
            commands::window::export_pdf,
            commands::workspace::get_directory_files,
            commands::workspace::workspace_exists,
            commands::workspace::create_file,
            commands::workspace::create_folder,
            commands::workspace::delete_file,
            commands::workspace::rename_file,
            commands::shell::open_external,
            commands::shell::open_link,
            commands::shell::get_system_fonts,
            commands::clipboard::write_temp_image,
            commands::clipboard::get_clipboard_file_path,
            commands::image::open_image_preview,
            commands::dialog::show_open_dialog,
            commands::dialog::show_image_unsaved_choice,
            commands::dialog::show_overwrite_confirm,
            // 增强功能
            commands::enhancements::export_html,
            commands::enhancements::generate_toc,
            commands::enhancements::search_files,
            commands::enhancements::export_epub,
            commands::enhancements::git_diff,
            commands::enhancements::check_links,
            watch_files,
            watch_directory,
            unwatch_directory,
            quit_and_install,
        ])
        .setup(|app| {
            let main_win = app.get_webview_window("main").unwrap();

            // ── macOS 关闭行为 ──
            let emitter = main_win.clone();
            main_win.on_window_event(move |event| {
                #[cfg(target_os = "macos")]
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = emitter.emit("close-confirm", ());
                }
            });

            // ── 文件拖放 ──
            let dw = main_win.clone();
            main_win.on_window_event(move |event| {
                if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                    let md_paths: Vec<String> = paths.iter()
                        .filter(|p| {
                            p.extension()
                                .map(|e| e == "md" || e == "markdown")
                                .unwrap_or(false)
                        })
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    if !md_paths.is_empty() {
                        let _ = dw.emit("file-dropped", md_paths);
                    }
                }
            });

            // ── 文件关联打开 ──
            let args: Vec<String> = std::env::args().collect();
            let md_files: Vec<String> = args.iter()
                .filter(|a| a.ends_with(".md") || a.ends_with(".markdown"))
                .map(|a| std::path::absolute(a).unwrap_or_else(|_| a.into()).to_string_lossy().to_string())
                .collect();

            for abs in &md_files {
                if let Ok(raw) = std::fs::read_to_string(abs) {
                    let normalized = raw.trim_start_matches('\u{FEFF}').replace("\r\n", "\n");
                    let payload = commands::file::file_result_from_raw(abs, &normalized, &raw);
                    let w2 = main_win.clone();
                    // 多次重试：前端加载可能需要 1-3 秒
                    for delay in [600, 1200, 2000] {
                        let w3 = w2.clone();
                        let p2 = serde_json::to_value(&payload).unwrap_or_default();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(delay));
                            let _ = w3.emit("open-file-at-launch", p2);
                        });
                    }
                }
            }

            // ── 单实例：监听新启动传入的文件路径 ──
            let w3 = main_win.clone();
            std::thread::spawn(move || {
                let mut buf = String::new();
                for mut stream in listener.incoming().flatten() {
                    buf.clear();
                    use std::io::Read;
                    if stream.read_to_string(&mut buf).is_ok() && !buf.is_empty() {
                        for file_path in buf.lines() {
                            let path = Path::new(file_path);
                            if path.extension().map(|e| e == "md" || e == "markdown").unwrap_or(false) {
                                if let Ok(raw) = std::fs::read_to_string(path) {
                                    let normalized = raw.trim_start_matches('\u{FEFF}').replace("\r\n", "\n");
                                    let payload = commands::file::file_result_from_raw(
                                        &path.to_string_lossy(), &normalized, &raw
                                    );
                                    let w4 = w3.clone();
                                    let p2 = serde_json::to_value(&payload).unwrap_or_default();
                                    std::thread::spawn(move || {
                                        let _ = w4.emit("open-file-at-launch", p2);
                                    });
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动 milkup 失败");
}

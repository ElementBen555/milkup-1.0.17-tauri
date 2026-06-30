//! 窗口管理命令
use serde::Serialize;
use tauri::Emitter;
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// 窗口操作：最小化/最大化/关闭，返回操作后窗口状态
#[tauri::command]
pub fn window_control(
    window: tauri::WebviewWindow,
    action: String,
) -> Result<serde_json::Value, String> {
    match action.as_str() {
        "minimize" => {
            window.minimize().map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "minimized": true }))
        }
        "maximize" => {
            let is_max = window.is_maximized().unwrap_or(false);
            if is_max {
                window.unmaximize().map_err(|e| e.to_string())?;
            } else {
                window.maximize().map_err(|e| e.to_string())?;
            }
            Ok(serde_json::json!({ "maximized": !is_max }))
        }
        "close" => {
            window.close().map_err(|e| e.to_string())?;
            Ok(serde_json::json!({}))
        }
        "is_maximized" => {
            let is_max = window.is_maximized().unwrap_or(false);
            let is_full = window.is_fullscreen().unwrap_or(false);
            Ok(serde_json::json!({ "maximized": is_max, "fullscreen": is_full }))
        }
        _ => Err(format!("未知窗口操作: {action}")),
    }
}

/// 关闭窗口（不经过保存确认）
#[tauri::command]
pub fn close_discard(window: tauri::WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// 获取窗口 bounds（位置 + 尺寸）
#[tauri::command]
pub fn get_window_bounds(
    window: tauri::WebviewWindow,
) -> Result<WindowBounds, String> {
    if let Ok(pos) = window.outer_position() {
        if let Ok(size) = window.outer_size() {
            return Ok(WindowBounds {
                x: pos.x as f64,
                y: pos.y as f64,
                width: size.width as f64,
                height: size.height as f64,
            });
        }
    }
    Err("无法获取窗口 bounds".into())
}

/// 设置窗口标题
#[tauri::command]
pub fn set_title(window: tauri::WebviewWindow, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())
}

/// 打开主题编辑器窗口
#[tauri::command]
pub fn open_theme_editor(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::WebviewUrl;
    use tauri::WebviewWindowBuilder;

    let label = "theme-editor";

    if let Some(existing) = app.get_webview_window(label) {
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, label, WebviewUrl::App("theme-editor.html".into()))
        .title("milkup - 主题编辑器")
        .inner_size(1000.0, 700.0)
        .min_inner_size(800.0, 600.0)
        .decorations(false)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Tab 拖拽分离：创建新窗口并传递 tab 数据
#[tauri::command]
pub fn tear_off_tab(app: tauri::AppHandle, tab_data: serde_json::Value, x: f64, y: f64) -> Result<String, String> {
    use tauri::WebviewUrl;
    use tauri::WebviewWindowBuilder;

    let label = format!("editor-{}", uuid::Uuid::new_v4());

    let win = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .title("milkup - Untitled")
        .inner_size(1000.0, 700.0)
        .position(x as f64, y as f64)
        .decorations(false)
        .visible(false) // 先隐藏，定位后再显示
        .build()
        .map_err(|e| format!("创建窗口失败: {e}"))?;

    // 延迟发送 tab 数据（等前端就绪）
    let w = win.clone();
    let a = app.clone();
    let l = label.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        let _ = w.show();
        let _ = w.set_focus();
        let _ = a.emit_to(&l, "open-file-at-launch", tab_data);
    });

    Ok(label)
}

/// 导出为 PDF（打开系统打印对话框，用户可另存为 PDF）
#[tauri::command]
pub fn export_pdf(window: tauri::WebviewWindow) -> Result<(), String> {
    use tauri::Manager;
    window.print().map_err(|e| e.to_string())
}
#[tauri::command]
pub fn merge_tab(app: tauri::AppHandle, target_label: String, tab_data: serde_json::Value) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&target_label) {
        app.emit_to(&target_label, "tab:merge-in", tab_data).map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("目标窗口不存在: {target_label}"))
    }
}

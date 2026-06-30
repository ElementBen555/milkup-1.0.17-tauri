//! Shell & 系统命令
use tauri_plugin_opener::OpenerExt;

/// 用系统默认应用打开外部链接
#[tauri::command]
pub fn open_external(app: tauri::AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// 智能打开链接（支持本地路径、URL、milkup:// 协议）
#[tauri::command]
pub fn open_link(
    app: tauri::AppHandle,
    href: String,
    current_file_path: Option<String>,
) -> Result<(), String> {
    // 本地相对路径：相对于当前 Markdown 文件解析
    if !href.starts_with("http://") && !href.starts_with("https://") {
        if let Some(base) = current_file_path {
            let base_path = std::path::Path::new(&base);
            if let Some(parent) = base_path.parent() {
                let resolved = parent.join(&href);
                if resolved.exists() {
                    return app
                        .opener()
                        .open_path(resolved.to_string_lossy(), None::<&str>)
                        .map_err(|e| e.to_string());
                }
            }
        }
    }
    app.opener()
        .open_url(href, None::<&str>)
        .map_err(|e| e.to_string())
}

/// 获取系统字体列表（用 fc-list 或 font-kit）
#[tauri::command]
pub fn get_system_fonts() -> Result<Vec<String>, String> {
    // Linux: 使用 fc-list
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("fc-list")
            .arg("--format=%{family}\n")
            .output()
            .map_err(|e| format!("无法执行 fc-list: {e}"))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut fonts: Vec<String> = stdout
                .lines()
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            fonts.sort();
            fonts.dedup();
            return Ok(fonts);
        }
        return Ok(vec![]);
    }

    // macOS: 使用 system_profiler
    #[cfg(target_os = "macos")]
    {
        // 简化实现，返回空列表
        return Ok(vec![]);
    }

    // Windows: 使用 DirectWrite
    #[cfg(target_os = "windows")]
    {
        // 简化实现，返回空列表
        return Ok(vec![]);
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Ok(vec![])
}

//! 文件 I/O 命令 — 对应原 ipcBridge.ts 中的文件操作
//!
//! 功能映射：
//!   dialog:openFile       → open_file()
//!   dialog:saveFile       → save_file()
//!   dialog:saveFileAs     → save_file_as()
//!   file:readByPath       → read_file_by_path()
//!   file:isReadOnly       → is_read_only()
//!   file:cleanupLocalImages → cleanup_local_images()

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 文件特征（BOM、换行符等），与前端 Tab.fileTraits 对应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTraits {
    pub has_bom: bool,
    pub line_ending: String, // "crlf" | "lf"
    pub has_trailing_newline: bool,
}

/// 文件读取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    pub file_path: String,
    pub content: String,
    pub file_traits: FileTraits,
    pub read_only: bool,
}

/// 保存文件参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileParams {
    pub file_path: Option<String>,
    pub content: String,
    #[serde(default)]
    pub file_traits: Option<FileTraits>,
    #[serde(default)]
    pub image_local_path: Option<String>,
    #[serde(default)]
    pub image_use_file_name_folder: Option<bool>,
}

/// 保存结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub file_path: String,
    pub content: String,
}

// ─── 工具函数（从原 fileFormat.ts 移植） ─────────────────

/// 规范化 Markdown：移除 BOM，CRLF → LF
fn normalize_markdown(text: &str) -> String {
    text.trim_start_matches('\u{FEFF}')
        .replace("\r\n", "\n")
}

/// 根据特征还原文件格式
fn restore_file_traits(content: &str, traits: &FileTraits) -> String {
    let mut result = content.to_string();

    if traits.line_ending == "crlf" {
        result = result.replace('\n', "\r\n");
    }

    if traits.has_trailing_newline {
        let eol = if traits.line_ending == "crlf" {
            "\r\n"
        } else {
            "\n"
        };
        if !result.ends_with(eol) {
            result.push_str(eol);
        }
    } else {
        while result.ends_with("\r\n") {
            result.truncate(result.len() - 2);
        }
        while result.ends_with('\n') {
            result.pop();
        }
    }

    if traits.has_bom {
        result.insert(0, '\u{FEFF}');
    }

    result
}

/// 检测文件特征
fn detect_file_traits(raw: &str) -> FileTraits {
    FileTraits {
        has_bom: raw.starts_with('\u{FEFF}'),
        line_ending: if raw.contains("\r\n") {
            "crlf".into()
        } else {
            "lf".into()
        },
        has_trailing_newline: raw.ends_with('\n'),
    }
}

/// 检查文件是否为只读
fn is_path_read_only(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mode = meta.permissions().mode();
            // 检查所有三个级别的写权限（owner/group/other）
            (mode & 0o222) == 0
        } else {
            false
        }
    }
    #[cfg(not(unix))]
    {
        if let Ok(meta) = fs::metadata(path) {
            meta.permissions().readonly()
        } else {
            false
        }
    }
}

// ─── Tauri 命令 ─────────────────────────────────────────

/// 打开文件（通过系统文件对话框）
#[tauri::command]
pub async fn open_file(app: tauri::AppHandle) -> Result<Option<FileResult>, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("Markdown", &["md", "markdown"])
        .blocking_pick_file();

    if let Some(file_path) = path {
        read_file_internal(&file_path.to_string())
    } else {
        Ok(None)
    }
}

/// 按路径读取文件（拖拽等场景，已有路径）
#[tauri::command]
pub fn read_file_by_path(file_path: String) -> Result<Option<FileResult>, String> {
    read_file_internal(&file_path)
}

/// 保存文件（若无路径则弹出保存对话框）
#[tauri::command]
pub async fn save_file(
    app: tauri::AppHandle,
    params: SaveFileParams,
) -> Result<Option<SaveResult>, String> {
    let file_path = match params.file_path {
        Some(p) => p,
        None => {
            // 无路径 → 弹出保存对话框（新建文件首次保存）
            use tauri_plugin_dialog::DialogExt;
            let path = app
                .dialog()
                .file()
                .add_filter("Markdown", &["md", "markdown"])
                .blocking_save_file();
            path.map(|p| p.to_string()).ok_or("用户取消保存")?
        }
    };
    let content = restore_file_traits(
        &params.content,
        &params.file_traits.unwrap_or(FileTraits {
            has_bom: false,
            line_ending: "lf".into(),
            has_trailing_newline: false,
        }),
    );

    fs::write(&file_path, &content).map_err(|e| format!("写入失败: {e}"))?;

    // 重读以获取精确内容（编辑器可能做了归一化）
    let raw = fs::read_to_string(&file_path).map_err(|e| format!("回读失败: {e}"))?;
    let normalized = normalize_markdown(&raw);

    Ok(Some(SaveResult {
        file_path,
        content: normalized,
    }))
}

/// 另存为（通过文件保存对话框）
#[tauri::command]
pub async fn save_file_as(
    app: tauri::AppHandle,
    params: SaveFileParams,
) -> Result<Option<SaveResult>, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("Markdown", &["md", "markdown"])
        .blocking_save_file();

    if let Some(save_path) = path {
        let new_params = SaveFileParams {
            file_path: Some(save_path.to_string()),
            ..params
        };
        save_file(app.clone(), new_params).await
    } else {
        Ok(None)
    }
}

/// 检查文件是否只读
#[tauri::command]
pub fn is_read_only(file_path: String) -> bool {
    is_path_read_only(Path::new(&file_path))
}

/// 清理内容中引用的本地临时图片
#[tauri::command]
pub fn cleanup_local_images(content: String) -> Result<(), String> {
    // 从 Markdown 内容中提取所有本地图片路径（![](...)）
    let re = regex_lite::Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap();
    let mut cleaned = 0;

    for cap in re.captures_iter(&content) {
        let path = cap[1].trim();
        // 跳过外部链接
        if path.starts_with("http://") || path.starts_with("https://") {
            continue;
        }
        // 尝试删除本地图片（忽略不存在的文件）
        if let Ok(_) = fs::remove_file(path) {
            cleaned += 1;
        }
    }

    if cleaned > 0 {
        eprintln!("[milkup] 清理了 {cleaned} 个临时图片");
    }
    Ok(())
}

// ─── 内部函数 ─────────────────────────────────────────

fn read_file_internal(file_path: &str) -> Result<Option<FileResult>, String> {
    let path = Path::new(file_path);

    if !path.exists() || !path.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("读取失败: {e}"))?;
    Ok(Some(file_result_from_raw(file_path, &normalize_markdown(&raw), &raw)))
}

/// 从原始文件数据构造 FileResult（供外部模块使用）
pub fn file_result_from_raw(file_path: &str, normalized_content: &str, raw: &str) -> FileResult {
    let file_traits = detect_file_traits(raw);
    let read_only = is_path_read_only(Path::new(file_path));
    FileResult {
        file_path: file_path.into(),
        content: normalized_content.into(),
        file_traits,
        read_only,
    }
}

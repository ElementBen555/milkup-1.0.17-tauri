//! 对话框命令

use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogOptions {
    #[serde(default)]
    pub properties: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub default_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogResult {
    pub file_paths: Vec<String>,
    pub canceled: bool,
}

/// 通用打开对话框（文件/目录选择器）
#[tauri::command]
pub async fn show_open_dialog(
    app: tauri::AppHandle,
    options: OpenDialogOptions,
) -> Result<OpenDialogResult, String> {
    let is_directory = options.properties.iter().any(|p| p == "openDirectory");

    if is_directory {
        let path = app.dialog().file().blocking_pick_folder();
        match path {
            Some(dir_path) => Ok(OpenDialogResult {
                file_paths: vec![dir_path.to_string()],
                canceled: false,
            }),
            None => Ok(OpenDialogResult {
                file_paths: vec![],
                canceled: true,
            }),
        }
    } else {
        let path = app
            .dialog()
            .file()
            .add_filter("Markdown", &["md", "markdown"])
            .add_filter("All Files", &["*"])
            .blocking_pick_file();
        match path {
            Some(file_path) => Ok(OpenDialogResult {
                file_paths: vec![file_path.to_string()],
                canceled: false,
            }),
            None => Ok(OpenDialogResult {
                file_paths: vec![],
                canceled: true,
            }),
        }
    }
}
/// 未保存文件时插入图片的选择对话框
/// 返回: "fallback" | "cancel"
#[tauri::command]
pub async fn show_image_unsaved_choice(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri_plugin_dialog::MessageDialogKind;
    let confirmed = app
        .dialog()
        .message("当前文件尚未保存，需要保存后才能使用文件名文件夹。")
        .title("保存文件")
        .kind(MessageDialogKind::Warning)
        .blocking_show();

    Ok(if confirmed { "fallback" } else { "cancel" }.into())
}

/// 外部修改文件覆盖确认对话框
#[tauri::command]
pub async fn show_overwrite_confirm(
    app: tauri::AppHandle,
    file_name: String,
) -> Result<String, String> {
    use tauri_plugin_dialog::MessageDialogKind;
    let confirmed = app
        .dialog()
        .message(format!("文件 \"{file_name}\" 已被外部修改。\n是否覆盖当前内容？"))
        .title("文件冲突")
        .kind(MessageDialogKind::Warning)
        .blocking_show();

    Ok(if confirmed { "overwrite" } else { "cancel" }.into())
}

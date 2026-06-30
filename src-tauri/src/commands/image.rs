//! 图片预览命令
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePreviewPayload {
    pub src: String,
    pub alt: Option<String>,
    pub items: Option<Vec<ImagePreviewItem>>,
    pub index: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImagePreviewItem {
    pub src: String,
    pub alt: Option<String>,
}

/// 打开图片预览窗口（复用系统默认图片查看器）
#[tauri::command]
pub fn open_image_preview(
    app: tauri::AppHandle,
    src: String,
    _alt: Option<String>,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(src, None::<&str>)
        .map_err(|e| e.to_string())
}

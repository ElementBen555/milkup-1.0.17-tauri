//! 剪贴板 & 图片命令
use serde::Deserialize;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTempImageParams {
    pub file: Vec<u8>,        // 图片二进制数据
    pub target_path: String,  // 图片存储目录（配置中的 localPath）
    pub current_file_path: Option<String>,
    pub file_name: String,
    pub mime_type: String,
    pub use_file_name_folder: bool,
}

/// 写入临时图片并返回路径
#[tauri::command]
pub fn write_temp_image(params: WriteTempImageParams) -> Result<String, String> {
    let extension = match params.mime_type.as_str() {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "png",
    };

    let filename = format!("{}.{extension}", Uuid::new_v4());
    let target_dir = Path::new(&params.target_path);

    // 确保目标目录存在
    fs::create_dir_all(target_dir).map_err(|e| format!("创建图片目录失败: {e}"))?;

    let output_path = target_dir.join(&filename);
    fs::write(&output_path, &params.file).map_err(|e| format!("写入图片失败: {e}"))?;

    Ok(output_path.to_string_lossy().into_owned())
}

/// 获取剪贴板中的文件路径（如有）
#[tauri::command]
pub async fn get_clipboard_file_path(
    _app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    // Tauri v2 clipboard API 主要用于文本，文件路径暂不支持
    // 保留接口供后续扩展
    Ok(None)
}

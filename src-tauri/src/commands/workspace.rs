//! 工作区命令（支持递归目录树）
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    #[serde(rename = "isDirectory")]
    pub is_dir: bool,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DirectoryEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileParams {
    pub dir_path: String,
    pub file_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderParams {
    pub dir_path: String,
    pub folder_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameParams {
    pub old_path: String,
    pub new_name: String,
}

const IGNORED_DIRS: &[&str] = &[
    ".git", ".vscode", ".idea", "node_modules", ".next", ".nuxt", "dist", "build", "coverage",
];

fn read_dir_recursive(dir_path: &Path, depth: u32) -> Result<Vec<DirectoryEntry>, String> {
    if depth > 5 { return Ok(vec![]); }

    let read_dir = fs::read_dir(dir_path).map_err(|e| format!("读取目录失败: {e}"))?;
    let mut entries = Vec::new();

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let path = entry.path().to_string_lossy().into_owned();

        if is_dir {
            if IGNORED_DIRS.contains(&name.as_str()) { continue; }
            let children = read_dir_recursive(&entry.path(), depth + 1).unwrap_or_default();
            entries.push(DirectoryEntry { name, is_dir: true, path, children: Some(children) });
        } else if name.ends_with(".md") || name.ends_with(".markdown") {
            entries.push(DirectoryEntry { name, is_dir: false, path, children: None });
        }
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(entries)
}

#[tauri::command]
pub fn get_directory_files(dir_path: String) -> Result<Vec<DirectoryEntry>, String> {
    let path = Path::new(&dir_path);
    if !path.is_dir() { return Err(format!("不是有效目录: {dir_path}")); }
    read_dir_recursive(path, 0)
}

#[tauri::command]
pub fn workspace_exists(dir_path: String) -> bool {
    Path::new(&dir_path).is_dir()
}

#[tauri::command]
pub fn create_file(params: CreateFileParams) -> Result<String, String> {
    let file_path = Path::new(&params.dir_path).join(&params.file_name);
    fs::write(&file_path, "").map_err(|e| format!("创建文件失败: {e}"))?;
    Ok(file_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn create_folder(params: CreateFolderParams) -> Result<String, String> {
    let dir_path = Path::new(&params.dir_path).join(&params.folder_name);
    fs::create_dir_all(&dir_path).map_err(|e| format!("创建文件夹失败: {e}"))?;
    Ok(dir_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn delete_file(file_path: String) -> Result<(), String> {
    fs::remove_file(&file_path).map_err(|e| format!("删除文件失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn rename_file(params: RenameParams) -> Result<String, String> {
    let old = Path::new(&params.old_path);
    let parent = old.parent().ok_or("无法获取父目录")?;
    let new_path = parent.join(&params.new_name);
    fs::rename(old, &new_path).map_err(|e| format!("重命名失败: {e}"))?;
    Ok(new_path.to_string_lossy().into_owned())
}

//! 增强功能：导出 HTML / 插入目录 / 全局搜索

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub file_path: String,
    pub line: usize,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub dir_path: String,
    pub query: String,
    #[serde(default)]
    pub case_sensitive: bool,
}

/// 导出 HTML 独立文件（含基础样式）
#[tauri::command]
pub fn export_html(
    app: tauri::AppHandle,
    content: String,
    title: Option<String>,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("HTML", &["html", "htm"])
        .blocking_save_file();

    let save_path = path.ok_or("用户取消")?;
    let title = title.unwrap_or_else(|| "Document".into());
    let html = format!(
        r#"<!DOCTYPE html><html lang="zh-CN"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><style>body{{max-width:860px;margin:40px auto;padding:0 24px;font:16px/1.8 system-ui,-apple-system,sans-serif;color:#333;background:#fff}}pre{{background:#f5f5f5;padding:16px;border-radius:6px;overflow-x:auto}}code{{font:14px/1.5 monospace}}blockquote{{border-left:3px solid #ddd;margin:0;padding:2px 16px;color:#666}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:8px}}img{{max-width:100%}}@media(prefers-color-scheme:dark){{body{{color:#c9d1d9;background:#0d1117}}pre{{background:#161b22}}blockquote{{color:#8b949e}}td,th{{border-color:#30363d}}}}</style></head><body>{content}</body></html>"#,
        content = content.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    );

    fs::write(save_path.to_string(), &html)
        .map_err(|e| format!("写入失败: {e}"))?;

    Ok(save_path.to_string())
}

/// 为当前 Markdown 内容生成目录 [TOC]
#[tauri::command]
pub fn generate_toc(content: String) -> String {
    let mut toc = String::new();
    for line in content.lines() {
        if let Some(heading) = line.strip_prefix('#') {
            let level = heading.chars().take_while(|c| *c == '#').count();
            if level > 0 && level <= 6 && heading.len() > level {
                let title = heading[level..].trim();
                let anchor = title.to_lowercase().replace(' ', "-");
                let indent = "  ".repeat(level - 1);
                toc.push_str(&format!("{indent}- [{title}](#{anchor})\n"));
            }
        }
    }
    if toc.is_empty() {
        "<!-- 未找到标题 -->".into()
    } else {
        format!("**目录**\n\n{toc}")
    }
}

/// 全局跨文件搜索
#[tauri::command]
pub fn search_files(params: SearchParams) -> Result<Vec<SearchMatch>, String> {
    let dir = Path::new(&params.dir_path);
    if !dir.is_dir() {
        return Err("无效目录".into());
    }

    let mut results = Vec::new();
    let query = if params.case_sensitive {
        params.query.clone()
    } else {
        params.query.to_lowercase()
    };

    search_dir(dir, &query, params.case_sensitive, &mut results)
        .map_err(|e| format!("搜索失败: {e}"))?;

    results.truncate(200); // 最多 200 条
    Ok(results)
}

fn search_dir(
    dir: &Path,
    query: &str,
    case_sensitive: bool,
    results: &mut Vec<SearchMatch>,
) -> Result<(), String> {
    if results.len() >= 200 { return Ok(()); }

    let entries = fs::read_dir(dir).map_err(|e| format!("{e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{e}"))?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "node_modules" { continue; }
            search_dir(&path, query, case_sensitive, results)?;
        } else if path.extension().map(|e| e == "md" || e == "markdown").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                for (i, line) in content.lines().enumerate() {
                    let check = if case_sensitive { line.to_string() } else { line.to_lowercase() };
                    if check.contains(query) {
                        let preview = if line.len() > 100 { &line[..100] } else { line };
                        results.push(SearchMatch {
                            file_path: path.to_string_lossy().into(),
                            line: i + 1,
                            content: preview.into(),
                        });
                        if results.len() >= 200 { return Ok(()); }
                    }
                }
            }
        }
    }
    Ok(())
}

/// 导出 EPUB 电子书（简化版：HTML 包装为 EPUB 容器）
#[tauri::command]
pub fn export_epub(
    app: tauri::AppHandle,
    content: String,
    title: Option<String>,
) -> Result<String, String> {
    use std::io::Write;
    use tauri_plugin_dialog::DialogExt;

    let path = app.dialog().file()
        .add_filter("EPUB", &["epub"])
        .blocking_save_file();
    let save_path = path.ok_or("用户取消")?;
    let title = title.unwrap_or_else(|| "Document".into());

    // 简化 EPUB：单文件 HTML 包装
    let html = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" lang="zh-CN">
<head><meta charset="UTF-8"/><title>{title}</title>
<style>body{{max-width:860px;margin:40px auto;padding:0 24px;font:16px/1.8 system-ui;color:#333}}pre{{background:#f5f5f5;padding:16px;border-radius:6px}}code{{font:14px/1.5 monospace}}blockquote{{border-left:3px solid #ddd;margin:0;padding:2px 16px;color:#666}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:8px}}img{{max-width:100%}}@media(prefers-color-scheme:dark){{body{{color:#c9d1d9;background:#0d1117}}pre{{background:#161b22}}blockquote{{color:#8b949e}}td,th{{border-color:#30363d}}}}</style></head>
<body>{content}</body></html>"#
    );

    fs::write(save_path.to_string(), &html).map_err(|e| format!("写入失败: {e}"))?;
    Ok(save_path.to_string())
}

/// Git 差异对比
#[tauri::command]
pub fn git_diff(file_path: String) -> Result<String, String> {
    let repo = git2::Repository::discover(&file_path)
        .map_err(|e| format!("未找到 Git 仓库: {e}"))?;
    let relative = repo.workdir()
        .and_then(|wd| pathdiff::diff_paths(&file_path, wd))
        .unwrap_or_else(|| std::path::PathBuf::from(&file_path));
    let head = repo.head().map_err(|e| format!("{e}"))?.peel_to_tree().map_err(|e| format!("{e}"))?;
    let disk_content = fs::read_to_string(&file_path).map_err(|e| format!("{e}"))?;
    let disk_blob = repo.blob(disk_content.as_bytes()).map_err(|e| format!("{e}"))?;
    let git_entry = head.get_path(&relative).map_err(|_| "文件未在 Git 中追踪".to_string())?;
    let git_obj = git_entry.to_object(&repo).map_err(|e| format!("{e}"))?;
    let git_blob = git_obj.peel_to_blob().map_err(|e| format!("{e}"))?;
    if git_blob.id() == disk_blob { return Ok("无差异".into()); }
    let git_content = String::from_utf8_lossy(git_blob.content());
    let disk_lines: Vec<&str> = disk_content.lines().collect();
    let git_lines: Vec<&str> = git_content.lines().collect();
    let mut diff = String::from("```diff\n");
    let max = git_lines.len().max(disk_lines.len());
    for i in 0..max {
        let g = git_lines.get(i); let d = disk_lines.get(i);
        if g == d { diff.push_str(&format!("  {}\n", g.unwrap_or(&""))); }
        else {
            if let Some(g) = g { diff.push_str(&format!("- {}\n", g)); }
            if let Some(d) = d { diff.push_str(&format!("+ {}\n", d)); }
        }
    }
    diff.push_str("```");
    Ok(diff)
}

/// 链接检查
#[tauri::command]
pub fn check_links(content: String) -> Result<Vec<String>, String> {
    let re = regex_lite::Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    let mut broken = Vec::new();
    for cap in re.captures_iter(&content) {
        let url = cap[2].trim();
        if url.starts_with("http://") || url.starts_with("https://") {
            match reqwest::blocking::Client::new()
                .head(url).timeout(std::time::Duration::from_secs(5)).send() {
                Ok(resp) if resp.status().is_success() => {},
                _ => broken.push(format!("{}: {}", &cap[1], url)),
            }
        }
    }
    Ok(broken)
}

/// 用户自定义模板的持久化存储。
///
/// 用户自建模板以 JSON 形式保存在应用数据目录下，与编译期内置模板分离管理。
use std::path::PathBuf;

use crate::error::AppError;
use crate::models::template::Template;

/// 用户自定义模板文件名。
const USER_TEMPLATES_FILE: &str = "user_templates.json";

/// 返回用户模板存储文件路径。
fn file_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join(USER_TEMPLATES_FILE)
}

/// 读取所有用户自定义模板；文件不存在或解析失败时返回空列表。
pub fn load(app_data_dir: &PathBuf) -> Vec<Template> {
    let path = file_path(app_data_dir);
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            log::warn!("读取用户模板失败，使用空列表: {e}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

/// 覆盖写入全部用户自定义模板。
pub fn save(app_data_dir: &PathBuf, templates: &[Template]) -> Result<(), AppError> {
    let path = file_path(app_data_dir);
    let json = serde_json::to_string_pretty(templates)?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// 新增或更新一个用户自定义模板（按 name 唯一键覆盖）。
pub fn upsert(app_data_dir: &PathBuf, template: Template) -> Result<(), AppError> {
    let mut templates = load(app_data_dir);
    if let Some(existing) = templates.iter_mut().find(|t| t.name == template.name) {
        *existing = template;
    } else {
        templates.push(template);
    }
    save(app_data_dir, &templates)
}

/// 删除指定名称的用户自定义模板。
///
/// 未找到同名模板时视为成功（幂等）。
pub fn delete(app_data_dir: &PathBuf, name: &str) -> Result<(), AppError> {
    let mut templates = load(app_data_dir);
    templates.retain(|t| t.name != name);
    save(app_data_dir, &templates)
}

/// 模板操作的 Tauri 命令处理器。
use tauri::State;

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::models::template::{Template, TemplateCategory};
use crate::services::app_settings::AppSettingsManager;
use crate::services::shell_config::ShellConfigManager;
use crate::services::template_library;
use crate::state::AppState;

/// 列出所有可用的内置模板，可选择按分类过滤。
///
/// # 参数
/// * `category` - 可选的分类过滤条件（字符串形式："git"、"docker"、"fileops"、"network"、"custom"）
#[tauri::command]
pub fn list_templates(category: Option<String>) -> Result<Vec<Template>, AppError> {
    let templates = template_library::get_builtin_templates();

    let filtered = match category {
        Some(cat) => {
            let target = match cat.to_lowercase().as_str() {
                "git" => TemplateCategory::Git,
                "docker" => TemplateCategory::Docker,
                "fileops" => TemplateCategory::FileOps,
                "network" => TemplateCategory::Network,
                "custom" => TemplateCategory::Custom,
                _ => return Ok(templates),
            };
            templates.into_iter().filter(|t| t.category == target).collect()
        },
        None => templates,
    };

    Ok(filtered)
}

/// 将选中的模板导入到当前 Shell 配置文件中。
///
/// # 参数
/// * `names` - 要导入的模板名称列表
#[tauri::command]
pub fn import_templates(state: State<'_, AppState>, names: Vec<String>) -> Result<usize, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);

    let templates = template_library::get_builtin_templates();
    let selected: Vec<&Template> = templates.iter().filter(|t| names.contains(&t.name)).collect();

    let mut imported_count = 0usize;
    for template in selected {
        let alias = Alias {
            name: template.name.clone(),
            command: template.command.clone(),
            tags: template.tags.clone(),
        };
        match ShellConfigManager::add_alias(&config_path, &alias) {
            Ok(()) => imported_count += 1,
            Err(AppError::AliasExists(_)) => {
                // 跳过已存在的别名
                continue;
            },
            Err(e) => return Err(e),
        }
    }

    Ok(imported_count)
}

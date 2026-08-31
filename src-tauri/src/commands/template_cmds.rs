/// 模板操作的 Tauri 命令处理器。
use tauri::State;

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::models::template::{Template, TemplateCategory};
use crate::services::app_settings::AppSettingsManager;
use crate::services::shell_config::ShellConfigManager;
use crate::services::template_library;
use crate::state::AppState;

/// 列出所有可用的模板（内置 + 用户自定义），可选择按分类过滤。
///
/// # 参数
/// * `category` - 可选的分类过滤条件（字符串形式："git"、"docker"、"fileops"、"network"、"custom"）
#[tauri::command]
pub fn list_templates(state: State<'_, AppState>, category: Option<String>) -> Result<Vec<Template>, AppError> {
    let mut templates = template_library::get_builtin_templates();
    // 合并用户自定义模板（持久化在应用数据目录）。
    templates.extend(crate::services::user_template_store::load(&state.app_data_dir));

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
/// * `names` - 要导入的模板名称列表（可包含内置与用户自定义模板）
#[tauri::command]
pub fn import_templates(state: State<'_, AppState>, names: Vec<String>) -> Result<ImportResult, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);

    // 合并内置与用户自定义模板，便于统一导入。
    let mut templates = template_library::get_builtin_templates();
    templates.extend(template_library::load_user_templates(&state.app_data_dir));

    let selected: Vec<Alias> = templates
        .iter()
        .filter(|t| names.contains(&t.name))
        .map(|t| Alias {
            name: t.name.clone(),
            command: t.command.clone(),
            tags: t.tags.clone(),
        })
        .collect();

    let _write_guard = state.config_write_lock();
    crate::services::config_backup::create_backup(&state.app_data_dir, &config_path)?;

    // 整批只写入一次；已存在的别名计入 skipped。
    let outcome = ShellConfigManager::add_aliases_batch(&config_path, &selected)?;
    for err in &outcome.errors {
        log::warn!("模板导入条目失败: {err}");
    }

    Ok(ImportResult { imported: outcome.success_count, skipped: outcome.skipped_count })
}

/// 导入模板的结果。
#[derive(serde::Serialize)]
pub struct ImportResult {
    /// 成功导入的模板数量。
    pub imported: usize,
    /// 因别名已存在而跳过的模板数量。
    pub skipped: usize,
}

/// 保存一个用户自定义模板（按名称覆盖）。
///
/// # 参数
/// * `template` - 要保存的模板（分类应固定为 `Custom`）
#[tauri::command]
pub fn save_template(state: State<'_, AppState>, template: Template) -> Result<(), AppError> {
    crate::services::user_template_store::upsert(&state.app_data_dir, template)
}

/// 删除一个用户自定义模板。
///
/// # 参数
/// * `name` - 要删除的模板名称
#[tauri::command]
pub fn delete_template(state: State<'_, AppState>, name: String) -> Result<(), AppError> {
    crate::services::user_template_store::delete(&state.app_data_dir, &name)
}

/// 列出所有用户自定义模板。
#[tauri::command]
pub fn list_user_templates(state: State<'_, AppState>) -> Result<Vec<Template>, AppError> {
    Ok(crate::services::user_template_store::load(&state.app_data_dir))
}

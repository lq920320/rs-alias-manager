/// 别名操作的 Tauri 命令处理器。
use tauri::State;

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::services::app_settings::AppSettingsManager;
use crate::services::shell_config::ShellConfigManager;
use crate::state::AppState;

/// 列出当前 Shell 配置文件中的所有别名。
///
/// 返回 `Alias` 对象列表。
#[tauri::command]
pub fn list_aliases(state: State<'_, AppState>) -> Result<Vec<Alias>, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);
    ShellConfigManager::list_aliases(&config_path)
}

/// 向当前 Shell 配置文件添加新别名。
///
/// # 参数
/// * `name` - 别名名称
/// * `command` - 别名所对应的命令
/// * `tags` - 可选的分组标签
#[tauri::command]
pub fn add_alias(
    state: State<'_, AppState>,
    name: String,
    command: String,
    tags: Option<Vec<String>>,
) -> Result<(), AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);
    let alias = Alias { name, command, tags: tags.unwrap_or_default() };
    ShellConfigManager::add_alias(&config_path, &alias)
}

/// 更新当前 Shell 配置文件中的已有别名。
///
/// # 参数
/// * `old_name` - 要更新的当前别名名称
/// * `name` - 新别名名称
/// * `command` - 新命令
/// * `tags` - 可选的新标签
#[tauri::command]
pub fn update_alias(
    state: State<'_, AppState>,
    old_name: String,
    name: String,
    command: String,
    tags: Option<Vec<String>>,
) -> Result<(), AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);
    let alias = Alias { name, command, tags: tags.unwrap_or_default() };
    ShellConfigManager::update_alias(&config_path, &old_name, &alias)
}

/// 从当前 Shell 配置文件中删除别名。
///
/// # 参数
/// * `name` - 要删除的别名名称
#[tauri::command]
pub fn delete_alias(state: State<'_, AppState>, name: String) -> Result<(), AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);
    ShellConfigManager::delete_alias(&config_path, &name)
}

/// 从环境变量检测当前 Shell 类型。
///
/// 返回包含 Shell 类型的应用设置，如 "bash"、"zsh" 或 "fish"。
#[tauri::command]
pub fn detect_shell(
    state: State<'_, AppState>,
) -> Result<crate::services::app_settings::AppSettings, AppError> {
    let mut settings = state.get_settings();
    settings.shell_type = ShellConfigManager::detect_shell();
    state.update_settings(&settings)?;
    Ok(settings)
}

/// 批量添加别名。
///
/// 一次性添加多个别名，减少文件 I/O 次数。
/// 返回成功添加的数量和失败的别名名称列表。
#[tauri::command]
pub fn batch_add_aliases(
    state: State<'_, AppState>,
    aliases: Vec<Alias>,
) -> Result<BatchResult, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);
    let mut success_count = 0usize;
    let mut errors = Vec::new();

    for alias in &aliases {
        match ShellConfigManager::add_alias(&config_path, alias) {
            Ok(()) => success_count += 1,
            Err(e) => errors.push(format!("{}: {}", alias.name, e)),
        }
    }

    Ok(BatchResult { success_count, errors })
}

/// 批量删除别名。
///
/// 一次性删除多个别名，减少文件 I/O 次数。
/// 返回成功删除的数量和失败的别名名称列表。
#[tauri::command]
pub fn batch_delete_aliases(
    state: State<'_, AppState>,
    names: Vec<String>,
) -> Result<BatchResult, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);
    let mut success_count = 0usize;
    let mut errors = Vec::new();

    for name in &names {
        match ShellConfigManager::delete_alias(&config_path, name) {
            Ok(()) => success_count += 1,
            Err(e) => errors.push(format!("{}: {}", name, e)),
        }
    }

    Ok(BatchResult { success_count, errors })
}

/// 批量操作的结果。
#[derive(serde::Serialize)]
pub struct BatchResult {
    /// 成功操作的数量。
    pub success_count: usize,
    /// 失败的错误信息列表。
    pub errors: Vec<String>,
}

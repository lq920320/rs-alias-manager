/// 别名操作的 Tauri 命令处理器。
use std::path::PathBuf;

use tauri::State;

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::services::app_settings::{AppSettings, AppSettingsManager};
use crate::services::shell_config::ShellConfigManager;

/// Tauri 持有的应用状态。
pub struct AppState {
    /// 应用数据目录路径，启动时设置。
    pub app_data_dir: PathBuf,
}

/// 列出当前 Shell 配置文件中的所有别名。
///
/// 返回 `Alias` 对象列表。
#[tauri::command]
pub fn list_aliases(state: State<'_, AppState>) -> Result<Vec<Alias>, AppError> {
    let settings = AppSettingsManager::load(&state.app_data_dir);
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
    let settings = AppSettingsManager::load(&state.app_data_dir);
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
    let settings = AppSettingsManager::load(&state.app_data_dir);
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
    let settings = AppSettingsManager::load(&state.app_data_dir);
    let config_path = AppSettingsManager::effective_config_path(&settings);
    ShellConfigManager::delete_alias(&config_path, &name)
}

/// 从环境变量检测当前 Shell 类型。
///
/// 返回包含 Shell 类型的应用设置，如 "bash"、"zsh" 或 "fish"。
#[tauri::command]
pub fn detect_shell(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    let mut settings = AppSettingsManager::load(&state.app_data_dir);
    settings.shell_type = ShellConfigManager::detect_shell();
    AppSettingsManager::save(&state.app_data_dir, &settings)?;
    Ok(settings)
}

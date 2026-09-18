/// 别名操作的 Tauri 命令处理器。
///
/// 所有会修改配置文件的命令都遵循同一流程：
/// 获取写锁 → 备份当前配置 → 执行变更，确保并发安全与可回滚。
use tauri::State;

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::services::app_settings::AppSettingsManager;
use crate::services::config_backup;
use crate::services::shell_config::{BatchOutcome, ShellConfigManager};
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

    let _write_guard = state.config_write_lock();

    // 新增别名前先判重名：命中则返回 AliasConflict，让前端弹「覆盖/取消」，
    // 而非由下层 add_alias_to_content 抛出 AliasExists（保留给内部场景）。
    let existing = ShellConfigManager::list_aliases(&config_path)?;
    if existing.iter().any(|a| a.name == name) {
        return Err(AppError::AliasConflict(name));
    }

    config_backup::create_backup(&state.app_data_dir, &config_path)?;
    let alias = Alias {
        name,
        command,
        tags: tags.unwrap_or_default(),
    };
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

    let _write_guard = state.config_write_lock();
    config_backup::create_backup(&state.app_data_dir, &config_path)?;

    let alias = Alias {
        name,
        command,
        tags: tags.unwrap_or_default(),
    };
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

    let _write_guard = state.config_write_lock();
    config_backup::create_backup(&state.app_data_dir, &config_path)?;

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
/// 整批只读写配置文件一次。
/// 返回成功添加的数量、跳过的数量与失败列表。
#[tauri::command]
pub fn batch_add_aliases(
    state: State<'_, AppState>,
    aliases: Vec<Alias>,
) -> Result<BatchOutcome, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);

    let _write_guard = state.config_write_lock();
    config_backup::create_backup(&state.app_data_dir, &config_path)?;

    ShellConfigManager::add_aliases_batch(&config_path, &aliases)
}

/// 批量更新别名（按名称原地覆盖）。
///
/// 整批只读写配置文件一次。
/// 返回成功更新的数量与失败列表。
#[tauri::command]
pub fn batch_update_aliases(
    state: State<'_, AppState>,
    aliases: Vec<Alias>,
) -> Result<BatchOutcome, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);

    let _write_guard = state.config_write_lock();
    config_backup::create_backup(&state.app_data_dir, &config_path)?;

    ShellConfigManager::update_aliases_batch(&config_path, &aliases)
}

/// 批量删除别名。
///
/// 整批只读写配置文件一次。
/// 返回成功删除的数量与失败列表。
#[tauri::command]
pub fn batch_delete_aliases(
    state: State<'_, AppState>,
    names: Vec<String>,
) -> Result<BatchOutcome, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);

    let _write_guard = state.config_write_lock();
    config_backup::create_backup(&state.app_data_dir, &config_path)?;

    ShellConfigManager::delete_aliases_batch(&config_path, &names)
}

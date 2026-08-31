/// 配置文件备份与回滚的 Tauri 命令处理器。
use tauri::State;

use crate::error::AppError;
use crate::services::app_settings::AppSettingsManager;
use crate::services::config_backup::{self, BackupEntry};
use crate::state::AppState;

/// 列出所有配置文件备份（最新在前）。
#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupEntry>, AppError> {
    Ok(config_backup::list_backups(&state.app_data_dir))
}

/// 将指定备份恢复到其原始配置文件路径。
///
/// 恢复前会先对当前配置文件创建一份备份，确保回滚操作本身可撤销。
///
/// # 参数
/// * `id` - 备份条目的唯一标识
#[tauri::command]
pub fn restore_backup(state: State<'_, AppState>, id: String) -> Result<BackupEntry, AppError> {
    let settings = state.get_settings();
    let config_path = AppSettingsManager::effective_config_path(&settings);

    let _write_guard = state.config_write_lock();
    config_backup::create_backup(&state.app_data_dir, &config_path)?;
    config_backup::restore_backup(&state.app_data_dir, &id)
}

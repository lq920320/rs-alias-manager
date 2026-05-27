/// 设置操作的 Tauri 命令处理器。
use tauri::State;

use crate::commands::alias_cmds::AppState;
use crate::error::AppError;
use crate::models::shell_type::ShellType;
use crate::services::app_settings::{AppSettings, AppSettingsManager};

/// 获取当前应用设置。
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    let settings = AppSettingsManager::load(&state.app_data_dir);
    Ok(settings)
}

/// 更新应用设置。
///
/// # 参数
/// * `shell_type` - 可选的新 Shell 类型（"bash"、"zsh"、"fish"）
/// * `custom_config_path` - 可选的自定义配置文件路径（空字符串表示清除）
/// * `auto_refresh` - 可选的自动刷新开关
#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    shell_type: Option<String>,
    custom_config_path: Option<String>,
    auto_refresh: Option<bool>,
) -> Result<AppSettings, AppError> {
    let mut settings = AppSettingsManager::load(&state.app_data_dir);

    if let Some(st) = shell_type {
        settings.shell_type = st.parse::<ShellType>().map_err(|e| AppError::ParseError(e))?;
    }

    if let Some(cp) = custom_config_path {
        settings.custom_config_path = if cp.is_empty() { None } else { Some(cp) };
    }

    if let Some(ar) = auto_refresh {
        settings.auto_refresh = ar;
    }

    AppSettingsManager::save(&state.app_data_dir, &settings)?;
    Ok(settings)
}

/// 根据当前设置获取生效的配置文件路径。
#[tauri::command]
pub fn get_config_file_path(state: State<'_, AppState>) -> Result<String, AppError> {
    let settings = AppSettingsManager::load(&state.app_data_dir);
    let path = AppSettingsManager::effective_config_path(&settings);
    Ok(path.to_string_lossy().to_string())
}

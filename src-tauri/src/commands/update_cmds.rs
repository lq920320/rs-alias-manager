/// 更新检查相关的 Tauri 命令处理器。
use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::models::shell_type::ShellType;
use crate::services::app_settings::AppSettingsManager;
use crate::services::shell_config::ShellConfigManager;
use crate::services::update_checker::{self, UpdateInfo};

/// 对当前配置文件执行 source，使别名变更尽快对新终端生效。
///
/// 该命令由前端在增删改别名成功、且 `instant_apply` 开启时调用。受 GUI 进程限制，
/// 仅对由本应用启动或之后新开的终端生效（详见 `ShellConfigManager::auto_source`）。
#[tauri::command]
pub fn auto_source(app: AppHandle, shell_type: ShellType) -> Result<(), AppError> {
    let settings = AppSettingsManager::load(
        &app.path()
            .app_data_dir()
            .map_err(|e| AppError::ConfigNotFound(e.to_string()))?,
    );
    let config_path = AppSettingsManager::effective_config_path(&settings);
    ShellConfigManager::auto_source(&app, &config_path, &shell_type)
}

/// 检查应用程序是否有可用更新。
///
/// 通过 GitHub Releases API 获取最新版本，与当前版本号进行比较。
/// 当前版本号从 `tauri.conf.json`（即打包时使用的权威版本）读取，
/// 而非 `CARGO_PKG_VERSION`，以避免后端 Cargo.toml 与发布版本不同步。
///
/// 返回包含版本信息和更新状态的 `UpdateInfo`。
#[tauri::command]
pub fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, AppError> {
    let current = app.package_info().version.to_string();
    update_checker::check_for_updates(&current)
}

/// 获取当前应用程序版本号。
///
/// 从 `tauri.conf.json` 的版本字段读取（通过 Tauri 的 `package_info`）。
#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

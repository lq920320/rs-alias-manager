/// 更新检查相关的 Tauri 命令处理器。
use tauri::AppHandle;

use crate::error::AppError;
use crate::services::update_checker::{self, UpdateInfo};

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

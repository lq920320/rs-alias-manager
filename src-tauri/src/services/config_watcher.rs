/// 配置文件变更监听。
///
/// 使用 `notify` 监听 Shell 配置文件（或其父目录）的变化，并在
/// 应用的 `auto_refresh` 开启时向前端广播 `config-changed` 事件。
use std::path::PathBuf;

use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

/// 配置变更事件的名称，前端通过 `listen("config-changed", ...)` 订阅。
pub const CONFIG_CHANGED_EVENT: &str = "config-changed";

/// 启动对指定配置文件所在目录的监听。
///
/// 返回的 `RecommendedWatcher` 需由调用方持有（drop 即停止监听）。
/// 监听始终运行；当变更发生且 `auto_refresh` 为 true 时，才向后端 emit 事件，
/// 由前端防抖后自动刷新别名列表。
pub fn start_watching(
    app: &AppHandle,
    config_path: &PathBuf,
) -> notify::Result<RecommendedWatcher> {
    let app_handle = app.clone();
    let target = config_path.clone();

    let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                // 仅关心写入/重命名/创建类事件。
                let is_write = matches!(
                    event.kind,
                    notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_)
                );
                if !is_write {
                    return;
                }

                // 事件可能来自父目录中的任意文件，过滤为目标配置文件。
                let matches_target = event
                    .paths
                    .iter()
                    .any(|p| p == &target || p.file_name() == target.file_name());
                if !matches_target {
                    return;
                }

                // 按 auto_refresh 开关决定是否广播（避免在关闭时频繁刷新）。
                let refresh = app_handle
                    .state::<crate::state::AppState>()
                    .get_settings()
                    .auto_refresh;
                if refresh {
                    let _ = app_handle.emit(CONFIG_CHANGED_EVENT, ());
                }
            },
            Err(e) => {
                log::warn!("配置文件监听出错: {e}");
            },
        }
    })?;

    // 监听父目录（直接监听单文件在部分平台不可靠）。
    let watch_dir = config_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}

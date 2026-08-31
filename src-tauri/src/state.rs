/// Tauri 后端的应用状态。
///
/// 集中管理后端共享状态，包括应用数据目录、缓存的设置，以及配置文件监听器。
use std::path::PathBuf;
use std::sync::Mutex;

use notify::RecommendedWatcher;
use tauri::AppHandle;

use crate::error::AppError;
use crate::services::app_settings::{AppSettings, AppSettingsManager};
use crate::services::config_watcher;

/// Tauri 管理的应用状态。
pub struct AppState {
    /// 应用数据目录路径，启动时设置。
    pub app_data_dir: PathBuf,
    /// Tauri 应用句柄，用于广播事件与执行命令。
    app_handle: AppHandle,
    /// 缓存的应用设置，避免每次命令调用都读取文件。
    cached_settings: Mutex<Option<AppSettings>>,
    /// 配置文件监听器句柄（drop 即停止）。
    watcher: Mutex<Option<RecommendedWatcher>>,
    /// 配置文件写锁：串行化所有读-改-写操作，防止并发命令互相覆盖。
    config_write_lock: Mutex<()>,
}

impl AppState {
    /// 创建新的应用状态。
    pub fn new(app_data_dir: PathBuf, app_handle: AppHandle) -> Self {
        Self {
            app_data_dir,
            app_handle,
            cached_settings: Mutex::new(None),
            watcher: Mutex::new(None),
            config_write_lock: Mutex::new(()),
        }
    }

    /// 获取配置文件写锁。
    ///
    /// 所有会修改 Shell 配置文件的命令在执行读-改-写前必须持有该锁。
    pub fn config_write_lock(&self) -> std::sync::MutexGuard<'_, ()> {
        self.config_write_lock.lock().unwrap()
    }

    /// 获取应用设置（优先使用缓存）。
    ///
    /// 如果缓存为空则从文件加载并缓存。
    pub fn get_settings(&self) -> AppSettings {
        let mut cache = self.cached_settings.lock().unwrap();
        if let Some(ref settings) = *cache {
            return settings.clone();
        }
        let settings = AppSettingsManager::load(&self.app_data_dir);
        *cache = Some(settings.clone());
        settings
    }

    /// 更新设置并刷新缓存。
    ///
    /// 若 `custom_config_path` 发生变化（影响监听目标），会重启配置监听器。
    pub fn update_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        let previous = self.get_settings();
        AppSettingsManager::save(&self.app_data_dir, settings)?;
        let mut cache = self.cached_settings.lock().unwrap();
        *cache = Some(settings.clone());
        drop(cache);

        if previous.custom_config_path != settings.custom_config_path {
            self.restart_watcher();
        }
        Ok(())
    }

    /// 使缓存失效，下次访问时重新从文件加载。
    pub fn invalidate_settings_cache(&self) {
        let mut cache = self.cached_settings.lock().unwrap();
        *cache = None;
    }

    /// 根据当前设置启动配置文件监听（若尚未启动）。
    pub fn start_watcher(&self) {
        let settings = self.get_settings();
        let config_path = AppSettingsManager::effective_config_path(&settings);
        match config_watcher::start_watching(&self.app_handle, &config_path) {
            Ok(watcher) => {
                let mut guard = self.watcher.lock().unwrap();
                *guard = Some(watcher);
            }
            Err(e) => log::warn!("启动配置文件监听失败: {e}"),
        }
    }

    /// 重启配置文件监听（配置路径变化时调用）。
    pub fn restart_watcher(&self) {
        // 先释放旧 watcher。
        {
            let mut guard = self.watcher.lock().unwrap();
            *guard = None;
        }
        self.start_watcher();
    }
}

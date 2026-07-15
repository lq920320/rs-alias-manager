/// Tauri 后端的应用状态。
///
/// 集中管理后端共享状态，包括应用数据目录和缓存的设置。
use std::path::PathBuf;
use std::sync::Mutex;

use crate::services::app_settings::{AppSettings, AppSettingsManager};

/// Tauri 管理的应用状态。
pub struct AppState {
    /// 应用数据目录路径，启动时设置。
    pub app_data_dir: PathBuf,
    /// 缓存的应用设置，避免每次命令调用都读取文件。
    cached_settings: Mutex<Option<AppSettings>>,
}

impl AppState {
    /// 创建新的应用状态。
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            cached_settings: Mutex::new(None),
        }
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
    pub fn update_settings(&self, settings: &AppSettings) -> Result<(), crate::error::AppError> {
        AppSettingsManager::save(&self.app_data_dir, settings)?;
        let mut cache = self.cached_settings.lock().unwrap();
        *cache = Some(settings.clone());
        Ok(())
    }

    /// 使缓存失效，下次访问时重新从文件加载。
    pub fn invalidate_settings_cache(&self) {
        let mut cache = self.cached_settings.lock().unwrap();
        *cache = None;
    }
}

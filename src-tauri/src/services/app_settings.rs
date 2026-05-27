/// 应用程序设置持久化。
///
/// 使用 Tauri 的 `app_data_dir` 将应用程序设置存储为 JSON。
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::shell_type::ShellType;

/// 应用程序设置模型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 当前选择的 Shell 类型。
    pub shell_type: ShellType,
    /// 自定义配置文件路径（覆盖 Shell 类型的默认路径）。
    #[serde(default)]
    pub custom_config_path: Option<String>,
    /// 配置文件变更时是否自动刷新别名列表。
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { shell_type: ShellType::from_env(), custom_config_path: None, auto_refresh: true }
    }
}

/// 管理应用程序设置的持久化。
pub struct AppSettingsManager;

impl AppSettingsManager {
    /// 返回 Tauri 应用数据目录下的设置文件路径。
    fn settings_path(app_data_dir: &PathBuf) -> PathBuf {
        app_data_dir.join("settings.json")
    }

    /// 从应用数据目录加载设置。
    ///
    /// 如果文件不存在或无法解析，则返回默认设置。
    pub fn load(app_data_dir: &PathBuf) -> AppSettings {
        let path = Self::settings_path(app_data_dir);
        if !path.exists() {
            return AppSettings::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => AppSettings::default(),
        }
    }

    /// 将设置保存到应用数据目录。
    pub fn save(app_data_dir: &PathBuf, settings: &AppSettings) -> Result<(), AppError> {
        fs::create_dir_all(app_data_dir)?;
        let path = Self::settings_path(app_data_dir);
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// 根据设置返回有效的配置文件路径。
    ///
    /// 如果设置了 `custom_config_path`，则覆盖 Shell 类型的默认路径。
    pub fn effective_config_path(settings: &AppSettings) -> PathBuf {
        settings
            .custom_config_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| settings.shell_type.config_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.auto_refresh, true);
        assert!(settings.custom_config_path.is_none());
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-settings");
        let _ = fs::create_dir_all(&dir);

        let settings = AppSettings {
            shell_type: ShellType::Zsh,
            custom_config_path: Some("/custom/path".to_string()),
            auto_refresh: false,
        };

        AppSettingsManager::save(&dir, &settings).unwrap();
        let loaded = AppSettingsManager::load(&dir);

        assert_eq!(loaded.shell_type, ShellType::Zsh);
        assert_eq!(loaded.custom_config_path, Some("/custom/path".to_string()));
        assert_eq!(loaded.auto_refresh, false);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_effective_config_path_custom() {
        let settings = AppSettings {
            shell_type: ShellType::Bash,
            custom_config_path: Some("/my/custom/bashrc".to_string()),
            auto_refresh: true,
        };
        assert_eq!(
            AppSettingsManager::effective_config_path(&settings),
            PathBuf::from("/my/custom/bashrc")
        );
    }

    #[test]
    fn test_effective_config_path_default() {
        let settings = AppSettings {
            shell_type: ShellType::Bash,
            custom_config_path: None,
            auto_refresh: true,
        };
        let path = AppSettingsManager::effective_config_path(&settings);
        assert!(path.to_string_lossy().ends_with(".bashrc"));
    }
}

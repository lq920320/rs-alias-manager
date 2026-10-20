/// 应用程序设置持久化。
///
/// 使用 Tauri 的 `app_data_dir` 将应用程序设置存储为 JSON。
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::shell_type::ShellType;
use crate::services::safe_writer::{quarantine_file, safe_write};

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
    /// 增删改别名后是否自动 source 配置文件（仅对新开/由本应用启动的终端生效）。
    #[serde(default)]
    pub instant_apply: bool,
    /// 界面语言（"en" 或 "zh"）。
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_auto_refresh() -> bool {
    true
}

fn default_locale() -> String {
    "en".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shell_type: ShellType::from_env(),
            custom_config_path: None,
            auto_refresh: true,
            instant_apply: false,
            locale: default_locale(),
        }
    }
}

/// 管理应用程序设置的持久化。
pub struct AppSettingsManager;

impl AppSettingsManager {
    /// 返回 Tauri 应用数据目录下的设置文件路径。
    fn settings_path(app_data_dir: &Path) -> PathBuf {
        app_data_dir.join("settings.json")
    }

    /// 从应用数据目录加载设置。
    ///
    /// 文件不存在时返回默认设置。读取或解析失败时，将损坏文件改名为
    /// `settings.json.corrupt-<时间戳>` 保留现场，并回退到默认设置。
    pub fn load(app_data_dir: &Path) -> AppSettings {
        let path = Self::settings_path(app_data_dir);
        if !path.exists() {
            return AppSettings::default();
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                log::warn!("读取设置文件失败，使用默认设置: {e}");
                return AppSettings::default();
            },
        };

        match serde_json::from_str(&content) {
            Ok(settings) => settings,
            Err(e) => {
                log::warn!("设置文件解析失败，已隔离损坏文件并回退默认设置: {e}");
                quarantine_file(&path);
                AppSettings::default()
            },
        }
    }

    /// 将设置以原子方式保存到应用数据目录。
    pub fn save(app_data_dir: &Path, settings: &AppSettings) -> Result<(), AppError> {
        fs::create_dir_all(app_data_dir)?;
        let path = Self::settings_path(app_data_dir);
        let content = serde_json::to_string_pretty(settings)?;
        safe_write(&path, &content)?;
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
        assert!(settings.auto_refresh);
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
            locale: "en".to_string(),
            instant_apply: false,
        };

        AppSettingsManager::save(&dir, &settings).unwrap();
        let loaded = AppSettingsManager::load(&dir);

        assert_eq!(loaded.shell_type, ShellType::Zsh);
        assert_eq!(loaded.custom_config_path, Some("/custom/path".to_string()));
        assert!(!loaded.auto_refresh);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_corrupt_settings_quarantined() {
        let dir = std::env::temp_dir().join(format!(
            "rs-alias-manager-test-settings-corrupt-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(&path, "{ broken").unwrap();

        // 解析失败时回退默认设置，且不丢损坏现场
        let loaded = AppSettingsManager::load(&dir);
        assert_eq!(loaded.shell_type, ShellType::from_env());
        assert!(!path.exists());
        let quarantined: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("settings.json.corrupt-")
            })
            .collect();
        assert_eq!(quarantined.len(), 1);
        assert_eq!(
            fs::read_to_string(quarantined[0].path()).unwrap(),
            "{ broken"
        );

        // 之后保存应能重建正常的设置文件
        AppSettingsManager::save(&dir, &AppSettings::default()).unwrap();
        assert!(path.exists());
        assert_eq!(
            AppSettingsManager::load(&dir).shell_type,
            ShellType::from_env()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_effective_config_path_custom() {
        let settings = AppSettings {
            shell_type: ShellType::Bash,
            custom_config_path: Some("/my/custom/bashrc".to_string()),
            auto_refresh: true,
            locale: "en".to_string(),
            instant_apply: false,
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
            locale: "en".to_string(),
            instant_apply: false,
        };
        let path = AppSettingsManager::effective_config_path(&settings);
        assert!(path.to_string_lossy().ends_with(".bashrc"));
    }
}

/// Shell 类型模型与检测逻辑。
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 表示支持的 Shell 类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    /// Bourne Again Shell（bash）。
    Bash,
    /// Z Shell（zsh）。
    Zsh,
    /// Friendly Interactive Shell（fish）。
    Fish,
}

impl ShellType {
    /// 返回该 Shell 类型对应的配置文件路径。
    ///
    /// - Bash: `~/.bashrc`
    /// - Zsh: `~/.zshrc`
    /// - Fish: `~/.config/fish/config.fish`
    pub fn config_path(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        match self {
            ShellType::Bash => home.join(".bashrc"),
            ShellType::Zsh => home.join(".zshrc"),
            ShellType::Fish => home.join(".config").join("fish").join("config.fish"),
        }
    }

    /// 从 `SHELL` 环境变量检测当前 Shell 类型。
    ///
    /// 检测失败时回退到 `ShellType::Bash`。
    pub fn from_env() -> Self {
        std::env::var("SHELL")
            .ok()
            .and_then(|shell| {
                let shell_path = PathBuf::from(&shell);
                let name = shell_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                match name {
                    "zsh" => Some(ShellType::Zsh),
                    "fish" => Some(ShellType::Fish),
                    "bash" => Some(ShellType::Bash),
                    _ => None,
                }
            })
            .unwrap_or(ShellType::Bash)
    }

    /// 返回用于显示的字符串标签。
    pub fn label(&self) -> &'static str {
        match self {
            ShellType::Bash => "Bash",
            ShellType::Zsh => "Zsh",
            ShellType::Fish => "Fish",
        }
    }

    /// 返回所有 Shell 类型变体。
    pub fn all() -> Vec<ShellType> {
        vec![ShellType::Bash, ShellType::Zsh, ShellType::Fish]
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_else(|_| "\"bash\"".to_string());
        // 去除 JSON 字符串的引号
        write!(f, "{}", s.trim_matches('"'))
    }
}

impl std::str::FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            _ => Err(format!("未知 Shell 类型: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_bash() {
        let path = ShellType::Bash.config_path();
        assert!(path.to_string_lossy().ends_with(".bashrc"));
    }

    #[test]
    fn test_config_path_zsh() {
        let path = ShellType::Zsh.config_path();
        assert!(path.to_string_lossy().ends_with(".zshrc"));
    }

    #[test]
    fn test_config_path_fish() {
        let path = ShellType::Fish.config_path();
        assert!(path.to_string_lossy().ends_with("config.fish"));
        assert!(path.to_string_lossy().contains("fish"));
    }

    #[test]
    fn test_from_env_with_zsh() {
        std::env::set_var("SHELL", "/bin/zsh");
        assert_eq!(ShellType::from_env(), ShellType::Zsh);
    }

    #[test]
    fn test_from_env_with_bash() {
        std::env::set_var("SHELL", "/bin/bash");
        assert_eq!(ShellType::from_env(), ShellType::Bash);
    }

    #[test]
    fn test_from_env_with_fish() {
        std::env::set_var("SHELL", "/usr/local/bin/fish");
        assert_eq!(ShellType::from_env(), ShellType::Fish);
    }

    #[test]
    fn test_from_env_unknown_shell_falls_back_to_bash() {
        std::env::set_var("SHELL", "/bin/csh");
        assert_eq!(ShellType::from_env(), ShellType::Bash);
    }

    #[test]
    fn test_from_str_bash() {
        assert_eq!("bash".parse::<ShellType>(), Ok(ShellType::Bash));
    }

    #[test]
    fn test_from_str_zsh() {
        assert_eq!("zsh".parse::<ShellType>(), Ok(ShellType::Zsh));
    }

    #[test]
    fn test_from_str_fish() {
        assert_eq!("fish".parse::<ShellType>(), Ok(ShellType::Fish));
    }

    #[test]
    fn test_from_str_case_insensitive() {
        assert_eq!("Bash".parse::<ShellType>(), Ok(ShellType::Bash));
        assert_eq!("ZSH".parse::<ShellType>(), Ok(ShellType::Zsh));
        assert_eq!("Fish".parse::<ShellType>(), Ok(ShellType::Fish));
    }

    #[test]
    fn test_from_str_unknown_returns_error() {
        let result = "unknown_shell".parse::<ShellType>();
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ShellType::Bash), "bash");
        assert_eq!(format!("{}", ShellType::Zsh), "zsh");
        assert_eq!(format!("{}", ShellType::Fish), "fish");
    }

    #[test]
    fn test_label() {
        assert_eq!(ShellType::Bash.label(), "Bash");
        assert_eq!(ShellType::Zsh.label(), "Zsh");
        assert_eq!(ShellType::Fish.label(), "Fish");
    }

    #[test]
    fn test_all() {
        let all = ShellType::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ShellType::Bash));
        assert!(all.contains(&ShellType::Zsh));
        assert!(all.contains(&ShellType::Fish));
    }

    #[test]
    fn test_serde_roundtrip() {
        let shell = ShellType::Zsh;
        let json = serde_json::to_string(&shell).unwrap();
        assert_eq!(json, "\"zsh\"");
        let parsed: ShellType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ShellType::Zsh);
    }
}

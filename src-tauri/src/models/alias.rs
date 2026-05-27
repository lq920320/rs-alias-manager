/// 别名数据模型。
use serde::{Deserialize, Serialize};

/// 表示一条 Shell 别名条目。
///
/// 别名将简短名称映射到较长的命令，例如 `alias gs='git status'`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alias {
    /// 别名名称（如 "gs"）。
    pub name: String,
    /// 别名所对应的命令（如 "git status"）。
    pub command: String,
    /// 可选标签，用于分组与过滤。
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Alias {
    /// 使用指定名称和命令创建新的 `Alias`。
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self { name: name.into(), command: command.into(), tags: Vec::new() }
    }

    /// 创建带有标签的 `Alias`。
    pub fn with_tags(
        name: impl Into<String>,
        command: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        Self { name: name.into(), command: command.into(), tags }
    }

    /// 验证别名名称。合法的名称必须：
    /// - 非空
    /// - 仅包含字母数字、连字符和下划线
    /// - 不能以连字符开头
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("别名名称不能为空".to_string());
        }
        if name.starts_with('-') {
            return Err("别名名称不能以连字符开头".to_string());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("别名名称只能包含字母数字、连字符和下划线".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_alias() {
        let alias = Alias::new("gs", "git status");
        assert_eq!(alias.name, "gs");
        assert_eq!(alias.command, "git status");
        assert!(alias.tags.is_empty());
    }

    #[test]
    fn test_with_tags() {
        let alias = Alias::with_tags("gs", "git status", vec!["git".to_string()]);
        assert_eq!(alias.tags, vec!["git"]);
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(Alias::validate_name("gs").is_ok());
        assert!(Alias::validate_name("my_alias").is_ok());
        assert!(Alias::validate_name("my-alias").is_ok());
        assert!(Alias::validate_name("alias123").is_ok());
        assert!(Alias::validate_name("_private").is_ok());
        assert!(Alias::validate_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(Alias::validate_name("").is_err());
    }

    #[test]
    fn test_validate_name_starts_with_hyphen() {
        assert!(Alias::validate_name("-alias").is_err());
    }

    #[test]
    fn test_validate_name_contains_spaces() {
        assert!(Alias::validate_name("my alias").is_err());
    }

    #[test]
    fn test_validate_name_contains_special_chars() {
        assert!(Alias::validate_name("alias!").is_err());
        assert!(Alias::validate_name("alias@name").is_err());
        assert!(Alias::validate_name("alias.name").is_err());
    }

    #[test]
    fn test_validate_name_starts_with_underscore() {
        assert!(Alias::validate_name("_alias").is_ok());
    }

    #[test]
    fn test_validate_name_only_numbers() {
        assert!(Alias::validate_name("123").is_ok());
    }

    #[test]
    fn test_alias_equality() {
        let a1 = Alias::new("gs", "git status");
        let a2 = Alias::new("gs", "git status");
        assert_eq!(a1, a2);
    }

    #[test]
    fn test_alias_inequality() {
        let a1 = Alias::new("gs", "git status");
        let a2 = Alias::new("gs", "git stash");
        assert_ne!(a1, a2);
    }

    #[test]
    fn test_serde_roundtrip() {
        let alias = Alias::with_tags("gs", "git status", vec!["git".to_string()]);
        let json = serde_json::to_string(&alias).unwrap();
        let parsed: Alias = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, alias);
    }
}

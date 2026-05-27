/// Shell 配置文件管理器。
///
/// 提供 Shell 配置文件（`.bashrc`、`.zshrc`、`config.fish`）中
/// 别名的 CRUD 操作。所有写入操作通过 `safe_writer` 原子执行。
use std::path::PathBuf;

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::models::shell_type::ShellType;
use crate::services::alias_parser::{
    add_alias_to_content, delete_alias_from_content, parse_aliases_from_content,
    update_alias_in_content,
};
use crate::services::safe_writer::{safe_read, safe_write};

/// 管理 Shell 配置文件的各类操作。
pub struct ShellConfigManager;

impl ShellConfigManager {
    /// 列出给定配置文件路径中的所有别名。
    ///
    /// 如果文件不存在则返回空向量。
    pub fn list_aliases(config_path: &PathBuf) -> Result<Vec<Alias>, AppError> {
        let content = safe_read(config_path)?;
        Ok(parse_aliases_from_content(&content))
    }

    /// 向配置文件添加新别名。
    ///
    /// 如果已存在同名别名则返回 `AppError::AliasExists`。
    /// 如果别名名称验证失败则返回 `AppError::InvalidAliasName`。
    pub fn add_alias(config_path: &PathBuf, alias: &Alias) -> Result<(), AppError> {
        Alias::validate_name(&alias.name).map_err(AppError::InvalidAliasName)?;
        let content = safe_read(config_path)?;
        let new_content = add_alias_to_content(&content, alias)?;
        safe_write(config_path, &new_content)?;
        Ok(())
    }

    /// 更新配置文件中的现有别名。
    ///
    /// `old_name` 标识要更新的别名。别名对象包含新值。
    /// 如果不存在 `old_name` 指定的别名，则返回 `AppError::AliasNotFound`。
    pub fn update_alias(
        config_path: &PathBuf,
        old_name: &str,
        alias: &Alias,
    ) -> Result<(), AppError> {
        Alias::validate_name(&alias.name).map_err(AppError::InvalidAliasName)?;
        let content = safe_read(config_path)?;
        let new_content = update_alias_in_content(&content, old_name, alias)?;
        safe_write(config_path, &new_content)?;
        Ok(())
    }

    /// 从配置文件中删除别名。
    ///
    /// 如果不存在指定名称的别名则返回 `AppError::AliasNotFound`。
    pub fn delete_alias(config_path: &PathBuf, name: &str) -> Result<(), AppError> {
        let content = safe_read(config_path)?;
        let new_content = delete_alias_from_content(&content, name)?;
        safe_write(config_path, &new_content)?;
        Ok(())
    }

    /// 根据给定 Shell 类型解析配置文件路径。
    pub fn get_config_path(shell_type: &ShellType) -> PathBuf {
        shell_type.config_path()
    }

    /// 从环境变量检测当前 Shell 类型。
    pub fn detect_shell() -> ShellType {
        ShellType::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_test_dir(test_name: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join("rs-alias-manager-test-shell-config")
            .join(format!("{}-{}", test_name, id));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_list_aliases_empty_file() {
        let dir = unique_test_dir("empty");
        let path = dir.join("empty_rc");
        fs::write(&path, "").unwrap();
        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert!(aliases.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_and_list_aliases() {
        let dir = unique_test_dir("add_list");
        let path = dir.join("test_rc");
        fs::write(&path, "# My shell config\n").unwrap();

        let alias = Alias::new("gs", "git status");
        ShellConfigManager::add_alias(&path, &alias).unwrap();

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "gs");
        assert_eq!(aliases[0].command, "git status");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_delete_alias() {
        let dir = unique_test_dir("delete");
        let path = dir.join("test_rc_del");
        fs::write(&path, "alias gs='git status'\nalias ll='ls -la'\n").unwrap();

        ShellConfigManager::delete_alias(&path, "gs").unwrap();

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "ll");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_update_alias() {
        let dir = unique_test_dir("update");
        let path = dir.join("test_rc_upd");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let updated = Alias::new("gs", "git status --short");
        ShellConfigManager::update_alias(&path, "gs", &updated).unwrap();

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases[0].command, "git status --short");

        let _ = fs::remove_dir_all(&dir);
    }

    // === 额外边界情况测试 ===

    #[test]
    fn test_list_aliases_nonexistent_file() {
        let dir = unique_test_dir("nonexistent");
        let path = dir.join("nonexistent_rc");
        // 不创建文件
        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert!(aliases.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_alias_duplicate_error() {
        let dir = unique_test_dir("dup");
        let path = dir.join("test_rc_dup");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let alias = Alias::new("gs", "git status --short");
        let result = ShellConfigManager::add_alias(&path, &alias);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_alias_invalid_name() {
        let dir = unique_test_dir("invalid");
        let path = dir.join("test_rc_invalid");
        fs::write(&path, "").unwrap();

        let alias = Alias::new("invalid name", "echo hello");
        let result = ShellConfigManager::add_alias(&path, &alias);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_delete_alias_not_found() {
        let dir = unique_test_dir("del_notfound");
        let path = dir.join("test_rc_notfound");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let result = ShellConfigManager::delete_alias(&path, "nonexistent");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_update_alias_not_found() {
        let dir = unique_test_dir("upd_notfound");
        let path = dir.join("test_rc_upd_notfound");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let alias = Alias::new("new", "echo hello");
        let result = ShellConfigManager::update_alias(&path, "nonexistent", &alias);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_update_alias_rename() {
        let dir = unique_test_dir("rename");
        let path = dir.join("test_rc_rename");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let renamed = Alias::new("gitstatus", "git status");
        ShellConfigManager::update_alias(&path, "gs", &renamed).unwrap();

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "gitstatus");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_full_crud_lifecycle() {
        let dir = unique_test_dir("lifecycle");
        let path = dir.join("test_rc_lifecycle");
        fs::write(&path, "# My config\n").unwrap();

        // 添加
        let alias1 = Alias::new("gs", "git status");
        ShellConfigManager::add_alias(&path, &alias1).unwrap();
        let alias2 = Alias::new("ll", "ls -la");
        ShellConfigManager::add_alias(&path, &alias2).unwrap();

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.len(), 2);

        // 更新
        let updated = Alias::new("gs", "git status --short");
        ShellConfigManager::update_alias(&path, "gs", &updated).unwrap();
        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.iter().find(|a| a.name == "gs").unwrap().command, "git status --short");

        // 删除
        ShellConfigManager::delete_alias(&path, "ll").unwrap();
        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "gs");

        // 验证非别名内容是否保留
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# My config"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_config_path() {
        let bash_path = ShellConfigManager::get_config_path(&ShellType::Bash);
        assert!(bash_path.to_string_lossy().ends_with(".bashrc"));

        let zsh_path = ShellConfigManager::get_config_path(&ShellType::Zsh);
        assert!(zsh_path.to_string_lossy().ends_with(".zshrc"));
    }
}

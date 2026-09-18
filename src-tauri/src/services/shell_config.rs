/// Shell 配置文件管理器。
///
/// 提供 Shell 配置文件（`.bashrc`、`.zshrc`、`config.fish`）中
/// 别名的 CRUD 操作。所有写入操作通过 `safe_writer` 原子执行。
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::alias::Alias;
use crate::models::shell_type::ShellType;
use crate::services::alias_parser::{
    add_alias_to_content, delete_alias_from_content, parse_aliases_from_content,
    rebuild_config_content, update_alias_in_content,
};
use crate::services::safe_writer::{safe_read, safe_write};

/// 管理 Shell 配置文件的各类操作。
pub struct ShellConfigManager;

impl ShellConfigManager {
    /// 列出给定配置文件路径中的所有别名。
    ///
    /// 如果文件不存在则返回空向量。
    pub fn list_aliases(config_path: &Path) -> Result<Vec<Alias>, AppError> {
        let content = safe_read(config_path)?;
        Ok(parse_aliases_from_content(&content))
    }

    /// 向配置文件添加新别名。
    ///
    /// 如果已存在同名别名则返回 `AppError::AliasExists`。
    /// 如果别名名称验证失败则返回 `AppError::InvalidAliasName`。
    pub fn add_alias(config_path: &Path, alias: &Alias) -> Result<(), AppError> {
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
    pub fn update_alias(config_path: &Path, old_name: &str, alias: &Alias) -> Result<(), AppError> {
        Alias::validate_name(&alias.name).map_err(AppError::InvalidAliasName)?;
        let content = safe_read(config_path)?;
        let new_content = update_alias_in_content(&content, old_name, alias)?;
        safe_write(config_path, &new_content)?;
        Ok(())
    }

    /// 从配置文件中删除别名。
    ///
    /// 如果不存在指定名称的别名则返回 `AppError::AliasNotFound`。
    pub fn delete_alias(config_path: &Path, name: &str) -> Result<(), AppError> {
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

    /// 批量添加别名：整批只读取与写入配置文件各一次。
    ///
    /// - 名称非法的条目计入 `errors`；
    /// - 与现有别名重名的条目计入 `skipped_count`；
    /// - 其余追加到文件末尾，计入 `success_count`。
    pub fn add_aliases_batch(
        config_path: &Path,
        aliases: &[Alias],
    ) -> Result<BatchOutcome, AppError> {
        let mut outcome = BatchOutcome::default();
        let content = safe_read(config_path)?;
        let mut current = parse_aliases_from_content(&content);

        let mut changed = false;
        for alias in aliases {
            if let Err(reason) = Alias::validate_name(&alias.name) {
                outcome.errors.push(format!(
                    "{}: {}",
                    alias.name,
                    AppError::InvalidAliasName(reason)
                ));
                continue;
            }
            if current.iter().any(|a| a.name == alias.name) {
                outcome.skipped_count += 1;
                continue;
            }
            current.push(alias.clone());
            outcome.success_count += 1;
            changed = true;
        }

        if changed {
            let new_content = rebuild_config_content(&content, &current);
            safe_write(config_path, &new_content)?;
        }
        Ok(outcome)
    }

    /// 批量删除别名：整批只读取与写入配置文件各一次。
    ///
    /// 不存在的名称计入 `errors`，成功移除的计入 `success_count`。
    pub fn delete_aliases_batch(
        config_path: &Path,
        names: &[String],
    ) -> Result<BatchOutcome, AppError> {
        let mut outcome = BatchOutcome::default();
        let content = safe_read(config_path)?;
        let current = parse_aliases_from_content(&content);
        let existing: std::collections::HashSet<String> =
            current.iter().map(|a| a.name.clone()).collect();

        let mut remaining = current;
        let mut changed = false;
        for name in names {
            if !existing.contains(name.as_str()) {
                outcome.errors.push(format!(
                    "{}: {}",
                    name,
                    AppError::AliasNotFound(name.clone())
                ));
                continue;
            }
            remaining.retain(|a| a.name != *name);
            outcome.success_count += 1;
            changed = true;
        }

        if changed {
            let new_content = rebuild_config_content(&content, &remaining);
            safe_write(config_path, &new_content)?;
        }
        Ok(outcome)
    }

    /// 批量更新别名：整批只读取与写入配置文件各一次。
    ///
    /// 按名称原地更新（保持条目在文件中的位置）：
    /// - 名称非法的条目计入 `errors`；
    /// - 不存在的名称计入 `errors`；
    /// - 成功更新的计入 `success_count`。
    pub fn update_aliases_batch(
        config_path: &Path,
        aliases: &[Alias],
    ) -> Result<BatchOutcome, AppError> {
        let mut outcome = BatchOutcome::default();
        let content = safe_read(config_path)?;
        let mut current = parse_aliases_from_content(&content);

        let mut changed = false;
        for alias in aliases {
            if let Err(reason) = Alias::validate_name(&alias.name) {
                outcome.errors.push(format!(
                    "{}: {}",
                    alias.name,
                    AppError::InvalidAliasName(reason)
                ));
                continue;
            }
            match current.iter_mut().find(|a| a.name == alias.name) {
                Some(existing) => {
                    if existing.command != alias.command || existing.tags != alias.tags {
                        existing.command = alias.command.clone();
                        existing.tags = alias.tags.clone();
                        changed = true;
                    }
                    outcome.success_count += 1;
                },
                None => {
                    outcome.errors.push(format!(
                        "{}: {}",
                        alias.name,
                        AppError::AliasNotFound(alias.name.clone())
                    ));
                },
            }
        }

        if changed {
            let new_content = rebuild_config_content(&content, &current);
            safe_write(config_path, &new_content)?;
        }
        Ok(outcome)
    }

    /// 对配置文件执行 `source`，使新别名在由本应用启动或后续新开的终端中生效。
    ///
    /// 注意：GUI 主进程并非 interactive shell，source 对已在运行的终端窗口不生效，
    /// 仅对经由本命令启动的子 shell 或之后新开的终端有效。
    pub fn auto_source(
        app: &tauri::AppHandle,
        config_path: &Path,
        shell_type: &ShellType,
    ) -> Result<(), AppError> {
        use tauri_plugin_shell::ShellExt;

        let path = config_path.to_string_lossy().to_string();
        let (program, args) = match shell_type {
            ShellType::Bash => ("bash", vec!["-c".to_string(), format!("source '{path}'")]),
            ShellType::Zsh => (
                "zsh",
                vec![
                    "-i".to_string(),
                    "-c".to_string(),
                    format!("source '{path}'"),
                ],
            ),
            // Fish 保留选项但本期不保证语义正确（已知限制）。
            ShellType::Fish => ("fish", vec!["-c".to_string(), format!("source '{path}'")]),
        };

        app.shell()
            .command(program)
            .args(args)
            .spawn()
            .map_err(|e| AppError::NetworkError(format!("执行 source 失败: {e}")))?;

        Ok(())
    }
}

/// 批量操作的结果。
#[derive(Debug, Default, serde::Serialize)]
pub struct BatchOutcome {
    /// 成功操作的条目数。
    pub success_count: usize,
    /// 因已存在而被跳过的条目数（仅批量添加）。
    pub skipped_count: usize,
    /// 失败条目的错误信息列表。
    pub errors: Vec<String>,
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
        assert_eq!(
            aliases.iter().find(|a| a.name == "gs").unwrap().command,
            "git status --short"
        );

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

    // === 批量操作测试 ===

    #[test]
    fn test_batch_add_single_write() {
        let dir = unique_test_dir("batch_add");
        let path = dir.join("test_rc_batch");
        fs::write(&path, "# header\n").unwrap();

        let aliases = vec![
            Alias::new("gs", "git status"),
            Alias::new("ll", "ls -la"),
            Alias::new("gp", "git push"),
        ];
        let outcome = ShellConfigManager::add_aliases_batch(&path, &aliases).unwrap();
        assert_eq!(outcome.success_count, 3);
        assert_eq!(outcome.skipped_count, 0);
        assert!(outcome.errors.is_empty());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# header"));
        assert!(content.contains("alias gs='git status'"));
        assert!(content.contains("alias ll='ls -la'"));
        assert!(content.contains("alias gp='git push'"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_add_skips_existing_and_invalid() {
        let dir = unique_test_dir("batch_add_skip");
        let path = dir.join("test_rc_batch_skip");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let aliases = vec![
            Alias::new("gs", "git status -s"), // 重名 → 跳过
            Alias::new("bad name", "echo hi"), // 非法名 → 错误
            Alias::new("ll", "ls -la"),        // 正常添加
        ];
        let outcome = ShellConfigManager::add_aliases_batch(&path, &aliases).unwrap();
        assert_eq!(outcome.success_count, 1);
        assert_eq!(outcome.skipped_count, 1);
        assert_eq!(outcome.errors.len(), 1);

        let content = fs::read_to_string(&path).unwrap();
        // 原有别名保持不变
        assert!(content.contains("alias gs='git status'"));
        assert!(!content.contains("git status -s"));
        assert!(content.contains("alias ll='ls -la'"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_add_no_change_no_write() {
        let dir = unique_test_dir("batch_add_noop");
        let path = dir.join("test_rc_batch_noop");
        fs::write(&path, "alias gs='git status'\n").unwrap();
        let before = fs::read_to_string(&path).unwrap();

        let outcome =
            ShellConfigManager::add_aliases_batch(&path, &[Alias::new("gs", "other")]).unwrap();
        assert_eq!(outcome.success_count, 0);
        assert_eq!(outcome.skipped_count, 1);
        assert_eq!(fs::read_to_string(&path).unwrap(), before);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_delete_single_write() {
        let dir = unique_test_dir("batch_del");
        let path = dir.join("test_rc_batch_del");
        fs::write(
            &path,
            "alias gs='git status'\nalias ll='ls -la'\nalias gp='git push'\n",
        )
        .unwrap();

        let outcome = ShellConfigManager::delete_aliases_batch(
            &path,
            &["gs".to_string(), "gp".to_string(), "missing".to_string()],
        )
        .unwrap();
        assert_eq!(outcome.success_count, 2);
        assert_eq!(outcome.errors.len(), 1);

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "ll");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_update_single_write_in_place() {
        let dir = unique_test_dir("batch_upd");
        let path = dir.join("test_rc_batch_upd");
        fs::write(
            &path,
            "# header\nalias gs='git status'\nalias ll='ls -la'\n",
        )
        .unwrap();

        let mut gs = Alias::new("gs", "git status --short");
        gs.tags = vec!["git".to_string()];
        let ll = Alias::new("ll", "ls -lah");
        let outcome = ShellConfigManager::update_aliases_batch(&path, &[gs, ll]).unwrap();
        assert_eq!(outcome.success_count, 2);
        assert!(outcome.errors.is_empty());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# header"));
        assert!(content.contains("alias gs='git status --short'"));
        assert!(content.contains("alias ll='ls -lah'"));
        // 原地更新：gs 仍在 ll 之前
        assert!(content.find("alias gs").unwrap() < content.find("alias ll").unwrap());

        let aliases = ShellConfigManager::list_aliases(&path).unwrap();
        let gs = aliases.iter().find(|a| a.name == "gs").unwrap();
        assert_eq!(gs.tags, vec!["git".to_string()]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_update_not_found_and_invalid() {
        let dir = unique_test_dir("batch_upd_err");
        let path = dir.join("test_rc_batch_upd_err");
        fs::write(&path, "alias gs='git status'\n").unwrap();

        let outcome = ShellConfigManager::update_aliases_batch(
            &path,
            &[
                Alias::new("missing", "echo hi"),
                Alias::new("bad name", "echo hi"),
            ],
        )
        .unwrap();
        assert_eq!(outcome.success_count, 0);
        assert_eq!(outcome.errors.len(), 2);

        // 无成功项，文件内容保持原样
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "alias gs='git status'\n");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_update_no_change_no_write() {
        let dir = unique_test_dir("batch_upd_noop");
        let path = dir.join("test_rc_batch_upd_noop");
        fs::write(&path, "alias gs='git status'\n").unwrap();
        let before = fs::read_to_string(&path).unwrap();

        let outcome =
            ShellConfigManager::update_aliases_batch(&path, &[Alias::new("gs", "git status")])
                .unwrap();
        assert_eq!(outcome.success_count, 1);
        assert!(outcome.errors.is_empty());
        assert_eq!(fs::read_to_string(&path).unwrap(), before);

        let _ = fs::remove_dir_all(&dir);
    }
}

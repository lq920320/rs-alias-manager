/// 安全文件写入工具。
///
/// 通过先写入临时文件再重命名到目标路径来实现原子写入。
/// 这可以防止写入过程中断导致的数据损坏。
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 若 `path` 是符号链接，返回其解析后的真实路径；否则原样返回。
///
/// 直接对链接路径做 rename 会把链接本身替换成普通文件，
/// 破坏 stow/chezmoi 这类用符号链接管理 dotfile 的布局。
fn resolve_write_target(path: &Path) -> PathBuf {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        },
        _ => path.to_path_buf(),
    }
}

/// 以原子方式将内容写入文件。
///
/// 1. 若目标为符号链接，先解析到真实文件。
/// 2. 将内容写入与目标同目录的临时文件。
/// 3. 复制原文件的权限到临时文件。
/// 4. 将临时文件重命名（在大多数文件系统上为原子操作）为 `path`。
///
/// 这确保目标文件永远不会处于部分写入状态。
pub fn safe_write(path: &Path, content: &str) -> Result<(), AppError> {
    let target = resolve_write_target(path);
    let target = target.as_path();
    let parent = target.parent().ok_or_else(|| {
        AppError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path has no parent directory: {}", target.display()),
        ))
    })?;

    // Ensure the parent directory exists
    fs::create_dir_all(parent)?;

    // Create a temporary file in the same directory
    let temp_name = format!(
        ".{}.tmp.{}",
        target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown"),
        std::process::id()
    );
    let temp_path = parent.join(&temp_name);

    // Write to the temporary file
    fs::write(&temp_path, content)?;

    // 保留原文件权限（如 600 的私有配置），避免临时文件默认权限泄露内容。
    if let Ok(meta) = fs::metadata(target) {
        fs::set_permissions(&temp_path, meta.permissions())?;
    }

    // Atomic rename
    fs::rename(&temp_path, target)?;

    Ok(())
}

/// 以 UTF-8 字符串形式读取文件内容。
///
/// 如果文件不存在则返回空字符串。
pub fn safe_read(path: &Path) -> Result<String, AppError> {
    if !path.exists() {
        return Ok(String::new());
    }
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// 将文件改名为 `<原名>.corrupt-<毫秒时间戳>` 并返回新路径。
///
/// 用于保留损坏文件的现场（如索引、设置解析失败），避免直接覆盖丢失证据。
/// 重命名失败时记录日志并返回 `None`。
pub fn quarantine_file(path: &Path) -> Option<PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file_name = path.file_name()?.to_string_lossy();
    let corrupt_path = path.with_file_name(format!("{file_name}.corrupt-{timestamp}"));
    match fs::rename(path, &corrupt_path) {
        Ok(()) => Some(corrupt_path),
        Err(e) => {
            log::warn!("隔离损坏文件 {} 失败: {e}", path.display());
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_safe_write_and_read() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-safe-write");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test_file.txt");

        safe_write(&file_path, "hello world").unwrap();
        let content = safe_read(&file_path).unwrap();
        assert_eq!(content, "hello world");

        // 清理
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_safe_read_nonexistent() {
        let path = std::env::temp_dir().join("nonexistent_file_12345.txt");
        let content = safe_read(&path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_safe_write_atomic() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-atomic");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("atomic_test.txt");

        safe_write(&file_path, "first").unwrap();
        safe_write(&file_path, "second").unwrap();

        let content = safe_read(&file_path).unwrap();
        assert_eq!(content, "second");

        // No temp file should remain
        assert!(!dir.join(".atomic_test.txt.tmp").exists());

        // 清理
        let _ = fs::remove_dir_all(&dir);
    }

    // === 额外边界情况测试 ===

    #[test]
    fn test_safe_write_empty_content() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-empty-content");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("empty.txt");

        safe_write(&file_path, "").unwrap();
        let content = safe_read(&file_path).unwrap();
        assert_eq!(content, "");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_safe_write_unicode_content() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-unicode");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("unicode.txt");

        safe_write(&file_path, "你好世界 🌍 Ñoño").unwrap();
        let content = safe_read(&file_path).unwrap();
        assert_eq!(content, "你好世界 🌍 Ñoño");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_safe_write_multiline_content() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-multiline");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("multiline.txt");

        let content = "line1\nline2\nline3\n";
        safe_write(&file_path, content).unwrap();
        let read_content = safe_read(&file_path).unwrap();
        assert_eq!(read_content, content);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_safe_write_creates_parent_directory() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-auto-dir");
        let nested_dir = dir.join("nested").join("deep");
        let file_path = nested_dir.join("file.txt");

        safe_write(&file_path, "nested content").unwrap();
        let content = safe_read(&file_path).unwrap();
        assert_eq!(content, "nested content");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_safe_write_overwrite_existing() {
        let dir = std::env::temp_dir().join("rs-alias-manager-test-overwrite");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("overwrite.txt");

        safe_write(&file_path, "original").unwrap();
        safe_write(&file_path, "replaced").unwrap();
        let content = safe_read(&file_path).unwrap();
        assert_eq!(content, "replaced");

        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn test_safe_write_preserves_symlink() {
        use std::os::unix::fs::symlink;

        let dir = std::env::temp_dir().join(format!(
            "rs-alias-manager-test-symlink-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let real_path = dir.join("real_rc");
        let link_path = dir.join(".zshrc");
        fs::write(&real_path, "alias a='1'\n").unwrap();
        symlink(&real_path, &link_path).unwrap();

        safe_write(&link_path, "alias b='2'\n").unwrap();

        // 链接仍是链接，内容落到真实文件上
        assert!(fs::symlink_metadata(&link_path)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_to_string(&real_path).unwrap(), "alias b='2'\n");

        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn test_safe_write_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!(
            "rs-alias-manager-test-perms-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("private_rc");
        fs::write(&file_path, "secret\n").unwrap();
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600)).unwrap();

        safe_write(&file_path, "updated secret\n").unwrap();

        let mode = fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "写入后应保留原文件权限");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_quarantine_file_preserves_content() {
        let dir = std::env::temp_dir().join(format!(
            "rs-alias-manager-test-quarantine-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("broken.json");
        fs::write(&path, "not json").unwrap();

        let quarantined = quarantine_file(&path).expect("应成功隔离");
        assert!(!path.exists());
        assert!(quarantined
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("broken.json.corrupt-"));
        assert_eq!(fs::read_to_string(&quarantined).unwrap(), "not json");

        let _ = fs::remove_dir_all(&dir);
    }
}

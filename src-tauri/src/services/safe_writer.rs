/// 安全文件写入工具。
///
/// 通过先写入临时文件再重命名到目标路径来实现原子写入。
/// 这可以防止写入过程中断导致的数据损坏。
use std::fs;
use std::path::Path;

use crate::error::AppError;

/// 以原子方式将内容写入文件。
///
/// 1. 将内容写入与 `path` 同目录的临时文件。
/// 2. 将临时文件重命名（在大多数文件系统上为原子操作）为 `path`。
///
/// 这确保目标文件永远不会处于部分写入状态。
pub fn safe_write(path: &Path, content: &str) -> Result<(), AppError> {
    let parent = path.parent().ok_or_else(|| {
        AppError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path has no parent directory: {}", path.display()),
        ))
    })?;

    // Ensure the parent directory exists
    fs::create_dir_all(parent)?;

    // Create a temporary file in the same directory
    let temp_name = format!(
        ".{}.tmp.{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
        std::process::id()
    );
    let temp_path = parent.join(&temp_name);

    // Write to the temporary file
    fs::write(&temp_path, content)?;

    // Atomic rename
    fs::rename(&temp_path, path)?;

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
}

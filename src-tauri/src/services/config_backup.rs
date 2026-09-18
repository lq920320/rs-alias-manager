/// Shell 配置文件的备份与回滚。
///
/// 每次修改配置文件前自动创建备份（保存在应用数据目录的 `backups/` 下），
/// 支持列出备份与一键恢复到指定快照。备份索引为 `backups/backups.json`，
/// 备份内容为 `<id>.bak` 文件。
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::services::safe_writer::{safe_read, safe_write};

/// 备份子目录名。
const BACKUP_DIR_NAME: &str = "backups";
/// 备份索引文件名。
const INDEX_FILE: &str = "backups.json";
/// 最多保留的备份数量，超出时删除最旧的。
const MAX_BACKUPS: usize = 20;

/// 单条备份元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    /// 唯一标识（毫秒时间戳 + 进程号）。
    pub id: String,
    /// 备份时配置文件的路径。
    pub original_path: String,
    /// 创建时间（Unix 毫秒）。
    pub created_at: u64,
    /// 备份内容字节数。
    pub size: u64,
}

fn backup_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(BACKUP_DIR_NAME)
}

fn index_path(app_data_dir: &Path) -> PathBuf {
    backup_dir(app_data_dir).join(INDEX_FILE)
}

/// 备份内容文件路径。
pub fn backup_file_path(app_data_dir: &Path, id: &str) -> PathBuf {
    backup_dir(app_data_dir).join(format!("{id}.bak"))
}

fn load_index(app_data_dir: &Path) -> Vec<BackupEntry> {
    let content = match safe_read(&index_path(app_data_dir)) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    if content.is_empty() {
        return Vec::new();
    }
    serde_json::from_str(&content).unwrap_or_else(|e| {
        log::warn!("备份索引解析失败，将重建: {e}");
        Vec::new()
    })
}

fn save_index(app_data_dir: &Path, entries: &[BackupEntry]) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(entries)?;
    safe_write(&index_path(app_data_dir), &json)?;
    Ok(())
}

/// 为指定配置文件创建一份备份。
///
/// 文件不存在或内容为空时返回 `Ok(None)`（首次添加别名等场景）。
/// 备份数量超过上限时自动清理最旧的备份。
pub fn create_backup(
    app_data_dir: &Path,
    config_path: &Path,
) -> Result<Option<BackupEntry>, AppError> {
    if !config_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read(config_path)?;
    if content.is_empty() {
        return Ok(None);
    }

    let dir = backup_dir(app_data_dir);
    std::fs::create_dir_all(&dir)?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let entry = BackupEntry {
        id: format!("{now_ms}-{}", std::process::id()),
        original_path: config_path.to_string_lossy().to_string(),
        created_at: now_ms,
        size: content.len() as u64,
    };

    std::fs::write(backup_file_path(app_data_dir, &entry.id), &content)?;

    let mut entries = load_index(app_data_dir);
    entries.push(entry.clone());
    // 超限时从最旧的开始清理（索引按创建顺序追加）
    while entries.len() > MAX_BACKUPS {
        let oldest = entries.remove(0);
        let _ = std::fs::remove_file(backup_file_path(app_data_dir, &oldest.id));
    }
    save_index(app_data_dir, &entries)?;

    Ok(Some(entry))
}

/// 列出所有备份（按创建时间倒序，最新在前）。
pub fn list_backups(app_data_dir: &Path) -> Vec<BackupEntry> {
    let mut entries = load_index(app_data_dir);
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    entries
}

/// 将指定备份恢复到其原始路径。
///
/// 调用方需先持有配置文件写锁；恢复动作本身也应先对当前状态创建备份。
pub fn restore_backup(app_data_dir: &Path, id: &str) -> Result<BackupEntry, AppError> {
    let entry = load_index(app_data_dir)
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::ConfigNotFound(format!("backup {id}")))?;

    let backup_path = backup_file_path(app_data_dir, &entry.id);
    if !backup_path.exists() {
        return Err(AppError::ConfigNotFound(format!("backup file for {id}")));
    }
    let content = std::fs::read_to_string(&backup_path)?;
    safe_write(Path::new(&entry.original_path), &content)?;
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_test_dir(name: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join("rs-alias-manager-test-backup")
            .join(format!("{}-{}", name, id));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_create_and_list_backup() {
        let dir = unique_test_dir("create");
        let app_data = dir.join("data");
        let config = dir.join(".zshrc");
        std::fs::write(&config, "alias gs='git status'\n").unwrap();

        let entry = create_backup(&app_data, &config).unwrap().unwrap();
        assert_eq!(entry.original_path, config.to_string_lossy());
        assert!(entry.size > 0);

        let list = list_backups(&app_data);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, entry.id);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_create_backup_missing_or_empty_file() {
        let dir = unique_test_dir("skip");
        let app_data = dir.join("data");

        let missing = dir.join("missing_rc");
        assert!(create_backup(&app_data, &missing).unwrap().is_none());

        let empty = dir.join("empty_rc");
        std::fs::write(&empty, "").unwrap();
        assert!(create_backup(&app_data, &empty).unwrap().is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_restore_backup() {
        let dir = unique_test_dir("restore");
        let app_data = dir.join("data");
        let config = dir.join(".zshrc");
        std::fs::write(&config, "alias gs='git status'\n").unwrap();

        let entry = create_backup(&app_data, &config).unwrap().unwrap();
        // 模拟文件被改坏
        std::fs::write(&config, "broken content\n").unwrap();

        let restored = restore_backup(&app_data, &entry.id).unwrap();
        assert_eq!(restored.id, entry.id);
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "alias gs='git status'\n"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_restore_missing_backup_errors() {
        let dir = unique_test_dir("restore_missing");
        let app_data = dir.join("data");
        let _ = std::fs::create_dir_all(&app_data);
        assert!(restore_backup(&app_data, "nope").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_backup_retention_prunes_oldest() {
        let dir = unique_test_dir("retention");
        let app_data = dir.join("data");
        let config = dir.join(".zshrc");

        for i in 0..(MAX_BACKUPS + 5) {
            std::fs::write(&config, format!("alias a{i}='cmd{i}'\n")).unwrap();
            // 保证时间戳递增，避免同毫秒 id 冲突
            std::thread::sleep(std::time::Duration::from_millis(2));
            create_backup(&app_data, &config).unwrap();
        }

        let list = list_backups(&app_data);
        assert_eq!(list.len(), MAX_BACKUPS);
        // 倒序：最新在前
        assert!(list.windows(2).all(|w| w[0].created_at >= w[1].created_at));
        // 最旧备份的文件应已被清理
        let oldest = list.last().unwrap();
        assert!(backup_file_path(&app_data, &oldest.id).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}

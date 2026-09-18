/// Tauri 后端的应用错误类型。
/// 使用 `thiserror` 实现符合人体工学的错误定义，这些错误同时实现了 `Serialize`
/// 以便可以从 Tauri 命令中返回。
///
/// 序列化结构为 `{ code, message, detail }`：
/// - `code`：稳定的机器可读错误码，前端按码分支处理并本地化展示；
/// - `message`：英文兜底文案（便于日志与开发排查）；
/// - `detail`：错误相关的上下文数据（如别名名称、文件路径等）。
use serde::Serialize;

/// 应用顶层错误类型。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// I/O 操作错误（文件读写等）。
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// 解析错误（格式不正确的别名行、非法格式等）。
    #[error("Parse error: {0}")]
    ParseError(String),

    /// 找不到请求的 Shell 配置文件。
    #[error("Config file not found: {0}")]
    ConfigNotFound(String),

    /// 指定名称的别名已存在。
    #[error("Alias already exists: {0}")]
    AliasExists(String),

    /// 添加别名时遇到重名冲突（前端据此弹「覆盖/取消」提示）。
    #[error("Alias \"{0}\" already exists")]
    AliasConflict(String),

    /// 指定名称的别名未找到。
    #[error("Alias not found: {0}")]
    AliasNotFound(String),

    /// 别名名称非法（包含空格、特殊字符等）。
    #[error("Invalid alias name: {0}")]
    InvalidAliasName(String),

    /// JSON 序列化/反序列化错误。
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 网络请求错误（检查更新等）。
    #[error("Network error: {0}")]
    NetworkError(String),

    /// GitHub API 触发了速率限制（HTTP 403）。
    #[error("GitHub API rate limited, please try again later")]
    RateLimited,

    /// 未找到仓库的发布记录（GitHub 返回 404）。
    #[error("No releases found for this repository")]
    ReleaseNotFound,
}

impl AppError {
    /// 返回稳定的机器可读错误码，供前端按码分支处理（不依赖文案匹配）。
    pub fn code(&self) -> &'static str {
        match self {
            AppError::IoError(_) => "io_error",
            AppError::ParseError(_) => "parse_error",
            AppError::ConfigNotFound(_) => "config_not_found",
            AppError::AliasExists(_) => "alias_exists",
            AppError::AliasConflict(_) => "alias_conflict",
            AppError::AliasNotFound(_) => "alias_not_found",
            AppError::InvalidAliasName(_) => "invalid_alias_name",
            AppError::JsonError(_) => "json_error",
            AppError::NetworkError(_) => "network_error",
            AppError::RateLimited => "rate_limited",
            AppError::ReleaseNotFound => "release_not_found",
        }
    }

    /// 返回错误相关的上下文数据（如别名名称、文件路径），供前端展示。
    pub fn detail(&self) -> Option<String> {
        match self {
            AppError::IoError(e) => Some(e.to_string()),
            AppError::ParseError(s) => Some(s.clone()),
            AppError::ConfigNotFound(p) => Some(p.clone()),
            AppError::AliasExists(n) => Some(n.clone()),
            AppError::AliasConflict(n) => Some(n.clone()),
            AppError::AliasNotFound(n) => Some(n.clone()),
            AppError::InvalidAliasName(n) => Some(n.clone()),
            AppError::JsonError(e) => Some(e.to_string()),
            AppError::NetworkError(s) => Some(s.clone()),
            AppError::RateLimited => None,
            AppError::ReleaseNotFound => None,
        }
    }
}

// 手动实现 Serialize，以便 AppError 可以从 Tauri 命令中返回。
// 序列化为 { code, message, detail } 结构：code 供前端逻辑分支与本地化，
// message 仅作兜底展示，detail 携带上下文。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 3)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("detail", &self.detail())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_structured() {
        let err = AppError::AliasConflict("gs".to_string());
        let json = serde_json::to_string(&err).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["code"], "alias_conflict");
        assert!(value["message"].as_str().unwrap().contains("gs"));
        assert_eq!(value["detail"], "gs");
    }

    #[test]
    fn test_detail_is_none_for_unit_variants() {
        assert!(AppError::RateLimited.detail().is_none());
        assert!(AppError::ReleaseNotFound.detail().is_none());
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(AppError::AliasExists("x".into()).code(), "alias_exists");
        assert_eq!(
            AppError::AliasNotFound("x".into()).code(),
            "alias_not_found"
        );
        assert_eq!(
            AppError::InvalidAliasName("x".into()).code(),
            "invalid_alias_name"
        );
        assert_eq!(AppError::RateLimited.code(), "rate_limited");
        assert_eq!(AppError::ReleaseNotFound.code(), "release_not_found");
    }
}

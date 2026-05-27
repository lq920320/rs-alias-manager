/// Tauri 后端的应用错误类型。
/// 使用 `thiserror` 实现符合人体工学的错误定义，这些错误同时实现了 `Serialize`
/// 以便可以从 Tauri 命令中返回。
use serde::Serialize;

/// 应用顶层错误类型。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// I/O 操作错误（文件读写等）。
    #[error("I/O 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 解析错误（格式不正确的别名行、非法格式等）。
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 找不到请求的 Shell 配置文件。
    #[error("配置文件未找到: {0}")]
    ConfigNotFound(String),

    /// 指定名称的别名已存在。
    #[error("别名已存在: {0}")]
    AliasExists(String),

    /// 指定名称的别名未找到。
    #[error("别名未找到: {0}")]
    AliasNotFound(String),

    /// 别名名称非法（包含空格、特殊字符等）。
    #[error("非法别名名称: {0}")]
    InvalidAliasName(String),

    /// JSON 序列化/反序列化错误。
    #[error("JSON 错误: {0}")]
    JsonError(#[from] serde_json::Error),
}

// 手动实现 Serialize，以便 AppError 可以从 Tauri 命令中返回。
// 我们将错误信息序列化为字符串表示形式。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

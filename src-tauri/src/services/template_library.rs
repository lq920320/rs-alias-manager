/// 内置模板库数据。
///
/// 从编译时嵌入的 JSON 文件加载模板数据，方便维护和扩展。
use crate::models::template::Template;

/// 编译时嵌入的模板 JSON 数据。
const TEMPLATES_JSON: &str = include_str!("templates.json");

/// 返回所有内置模板。
///
/// 从编译时嵌入的 JSON 反序列化。
/// 如果 JSON 解析失败（不应该发生）则返回空列表。
pub fn get_builtin_templates() -> Vec<Template> {
    serde_json::from_str(TEMPLATES_JSON).unwrap_or_else(|e| {
        log::error!("Failed to parse builtin templates: {}", e);
        Vec::new()
    })
}

/// 模板数据模型。
use serde::{Deserialize, Serialize};

/// 模板的分类类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    /// Git 相关别名。
    Git,
    /// Docker 相关别名。
    Docker,
    /// 文件操作别名。
    FileOps,
    /// 网络相关别名。
    Network,
    /// 用户自定义别名。
    Custom,
}

impl TemplateCategory {
    /// 返回该分类的显示标签。
    pub fn label(&self) -> &'static str {
        match self {
            TemplateCategory::Git => "Git",
            TemplateCategory::Docker => "Docker",
            TemplateCategory::FileOps => "文件操作",
            TemplateCategory::Network => "网络",
            TemplateCategory::Custom => "自定义",
        }
    }

    /// 返回所有分类变体。
    pub fn all() -> Vec<TemplateCategory> {
        vec![
            TemplateCategory::Git,
            TemplateCategory::Docker,
            TemplateCategory::FileOps,
            TemplateCategory::Network,
            TemplateCategory::Custom,
        ]
    }
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// 表示一条可导入用户 Shell 配置的模板别名。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Template {
    /// 别名名称。
    pub name: String,
    /// 别名所对应的命令。
    pub command: String,
    /// 别名功能的简短描述。
    pub description: String,
    /// 该模板所属分类。
    pub category: TemplateCategory,
    /// 可选标签。
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Template {
    /// 创建新的 `Template`。
    pub fn new(
        name: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
        category: TemplateCategory,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: description.into(),
            category,
            tags: Vec::new(),
        }
    }

    /// 创建带有标签的 `Template`。
    pub fn with_tags(
        name: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
        category: TemplateCategory,
        tags: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: description.into(),
            category,
            tags,
        }
    }
}

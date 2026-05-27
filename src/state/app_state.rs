/// 使用 Leptos signals 的全局应用程序状态。
///
/// 持有整个应用程序所需的所有响应式状态：
/// - 当前别名列表
/// - Shell 类型和配置路径
/// - 用于过滤的搜索查询
/// - 多选操作的选择状态
/// - 加载和错误状态
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// 表示前端的 Shell 别名。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alias {
    /// 别名名称。
    pub name: String,
    /// 别名展开后的命令。
    pub command: String,
    /// 用于分组和过滤的标签。
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 表示 Shell 类型。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    /// Bourne Again Shell。
    Bash,
    /// Z Shell。
    Zsh,
    /// Friendly Interactive Shell。
    Fish,
}

impl ShellType {
    /// 返回此 Shell 类型的显示标签。
    pub fn label(&self) -> &'static str {
        match self {
            ShellType::Bash => "Bash",
            ShellType::Zsh => "Zsh",
            ShellType::Fish => "Fish",
        }
    }

    /// 返回所有 Shell 类型变体。
    pub fn all() -> Vec<ShellType> {
        vec![ShellType::Bash, ShellType::Zsh, ShellType::Fish]
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_else(|_| "\"bash\"".to_string());
        write!(f, "{}", s.trim_matches('"'))
    }
}

impl std::str::FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            _ => Err(format!("Unknown shell type: {}", s)),
        }
    }
}

/// 模板分类类型。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    /// Git 相关的别名。
    Git,
    /// Docker 相关的别名。
    Docker,
    /// 文件操作别名。
    #[serde(rename = "fileops")]
    FileOps,
    /// 网络相关别名。
    Network,
    /// 用户自定义别名。
    Custom,
}

impl TemplateCategory {
    /// 返回此分类的显示标签。
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
        let s = serde_json::to_string(self).unwrap_or_else(|_| "\"git\"".to_string());
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// 表示可以导入的模板别名。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Template {
    /// 别名名称。
    pub name: String,
    /// 别名展开后的命令。
    pub command: String,
    /// 简短描述。
    pub description: String,
    /// 分类。
    pub category: TemplateCategory,
    /// 可选标签。
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 应用程序设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 当前选择的 Shell 类型。
    pub shell_type: ShellType,
    /// 自定义配置文件路径。
    #[serde(default)]
    pub custom_config_path: Option<String>,
    /// 是否自动刷新别名列表。
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { shell_type: ShellType::Bash, custom_config_path: None, auto_refresh: true }
    }
}

/// 全局应用程序状态上下文。
#[derive(Debug, Clone, Copy)]
pub struct AppState {
    /// 所有别名的列表。
    pub aliases: ReadSignal<Vec<Alias>>,
    /// 别名列表的设置器。
    pub set_aliases: WriteSignal<Vec<Alias>>,
    /// 当前 Shell 类型。
    pub shell_type: ReadSignal<ShellType>,
    /// Shell 类型的设置器。
    pub set_shell_type: WriteSignal<ShellType>,
    /// 当前配置文件路径。
    pub config_path: ReadSignal<String>,
    /// 配置文件路径的设置器。
    pub set_config_path: WriteSignal<String>,
    /// 过滤别名的搜索查询。
    pub search_query: ReadSignal<String>,
    /// 搜索查询的设置器。
    pub set_search_query: WriteSignal<String>,
    /// 选中的别名名称集合（用于多选操作）。
    pub selected_aliases: ReadSignal<Vec<String>>,
    /// 选中别名的设置器。
    pub set_selected_aliases: WriteSignal<Vec<String>>,
    /// 是否正在进行加载操作。
    pub loading: ReadSignal<bool>,
    /// 加载状态的设置器。
    pub set_loading: WriteSignal<bool>,
    /// 当前错误消息（如果有的话）。
    pub error_message: ReadSignal<Option<String>>,
    /// 错误消息的设置器。
    pub set_error_message: WriteSignal<Option<String>>,
    /// 应用程序设置。
    pub settings: ReadSignal<AppSettings>,
    /// 应用程序设置的设置器。
    pub set_settings: WriteSignal<AppSettings>,
}

impl AppState {
    /// 使用默认值创建新的 `AppState`。
    pub fn new() -> Self {
        let (aliases, set_aliases) = signal(Vec::new());
        let (shell_type, set_shell_type) = signal(ShellType::Bash);
        let (config_path, set_config_path) = signal(String::new());
        let (search_query, set_search_query) = signal(String::new());
        let (selected_aliases, set_selected_aliases) = signal(Vec::new());
        let (loading, set_loading) = signal(false);
        let (error_message, set_error_message) = signal(None::<String>);
        let (settings, set_settings) = signal(AppSettings::default());

        Self {
            aliases,
            set_aliases,
            shell_type,
            set_shell_type,
            config_path,
            set_config_path,
            search_query,
            set_search_query,
            selected_aliases,
            set_selected_aliases,
            loading,
            set_loading,
            error_message,
            set_error_message,
            settings,
            set_settings,
        }
    }

    /// 根据当前搜索查询返回过滤后的别名列表。
    pub fn filtered_aliases(&self) -> Vec<Alias> {
        let query = self.search_query.get();
        let aliases = self.aliases.get();

        if query.is_empty() {
            return aliases;
        }

        let lower_query = query.to_lowercase();
        aliases
            .into_iter()
            .filter(|a| {
                a.name.to_lowercase().contains(&lower_query)
                    || a.command.to_lowercase().contains(&lower_query)
                    || a.tags.iter().any(|t| t.to_lowercase().contains(&lower_query))
            })
            .collect()
    }
}

/// 向子组件提供 `AppState`。
#[component]
pub fn AppstateProvider(children: Children) -> impl IntoView {
    let state = AppState::new();
    provide_context(state);
    children()
}

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

use crate::i18n::Locale;

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
        match self {
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Fish => write!(f, "fish"),
        }
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
    /// Returns the display label for this category (i18n-aware).
    pub fn label(&self) -> String {
        match self {
            TemplateCategory::Git => "Git".to_string(),
            TemplateCategory::Docker => "Docker".to_string(),
            TemplateCategory::FileOps => crate::i18n::t("category.file_ops"),
            TemplateCategory::Network => crate::i18n::t("category.network"),
            TemplateCategory::Custom => crate::i18n::t("category.custom"),
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
        match self {
            TemplateCategory::Git => write!(f, "git"),
            TemplateCategory::Docker => write!(f, "docker"),
            TemplateCategory::FileOps => write!(f, "fileops"),
            TemplateCategory::Network => write!(f, "network"),
            TemplateCategory::Custom => write!(f, "custom"),
        }
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
    /// 界面语言。
    #[serde(default = "default_locale_str")]
    pub locale: String,
}

fn default_auto_refresh() -> bool {
    true
}

fn default_locale_str() -> String {
    "en".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shell_type: ShellType::Bash,
            custom_config_path: None,
            auto_refresh: true,
            locale: default_locale_str(),
        }
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
    /// 当前成功消息（如果有的话）。
    pub success_message: ReadSignal<Option<String>>,
    /// 成功消息的设置器。
    pub set_success_message: WriteSignal<Option<String>>,
    /// 应用程序设置。
    pub settings: ReadSignal<AppSettings>,
    /// 应用程序设置的设置器。
    pub set_settings: WriteSignal<AppSettings>,
    /// 当前界面语言。
    pub locale: ReadSignal<Locale>,
    /// 界面语言的设置器。
    pub set_locale: WriteSignal<Locale>,
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
        let (success_message, set_success_message) = signal(None::<String>);
        let (settings, set_settings) = signal(AppSettings::default());
        let (locale, set_locale) = signal(Locale::En);

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
            success_message,
            set_success_message,
            settings,
            set_settings,
            locale,
            set_locale,
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

    /// 加载设置和配置路径（多页面共用逻辑）。
    pub fn load_settings_and_config(&self) {
        use leptos::task::spawn_local;
        let state = *self;
        spawn_local(async move {
            match crate::api::commands::get_settings().await {
                Ok(settings) => {
                    state.set_shell_type.set(settings.shell_type);
                    // Update locale from settings
                    if let Ok(loc) = settings.locale.parse::<Locale>() {
                        state.set_locale.set(loc);
                    }
                    state.set_settings.set(settings);
                },
                Err(e) => {
                    log::warn!("Failed to load settings: {}", e);
                },
            }
            match crate::api::commands::get_config_file_path().await {
                Ok(path) => {
                    state.set_config_path.set(path);
                },
                Err(e) => {
                    log::warn!("Failed to get config path: {}", e);
                },
            }
        });
    }

    /// 加载别名列表。
    pub fn load_aliases(&self) {
        use leptos::task::spawn_local;
        let state = *self;
        spawn_local(async move {
            state.set_loading.set(true);
            state.set_error_message.set(None);
            match crate::api::commands::list_aliases().await {
                Ok(aliases) => {
                    state.set_aliases.set(aliases);
                },
                Err(e) => {
                    state.set_error_message.set(Some(e));
                },
            }
            state.set_loading.set(false);
        });
    }
}

/// 向子组件提供 `AppState`。
#[component]
pub fn AppstateProvider(children: Children) -> impl IntoView {
    let state = AppState::new();
    provide_context(state);
    // Provide locale signal separately for i18n::t() function access
    provide_context(state.locale);

    // Update HTML lang attribute when locale changes
    Effect::new(move || {
        let locale = state.locale.get();
        crate::utils::set_html_lang(&locale.to_string());
    });

    children()
}

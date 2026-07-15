/// Internationalization (i18n) module.
///
/// Provides locale management and translation lookup via Leptos context.
/// Supports English (default) and Chinese.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English (default).
    En,
    /// Chinese (Simplified).
    Zh,
}

impl Default for Locale {
    fn default() -> Self {
        Locale::En
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::En => write!(f, "en"),
            Locale::Zh => write!(f, "zh"),
        }
    }
}

impl std::str::FromStr for Locale {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" => Ok(Locale::En),
            "zh" => Ok(Locale::Zh),
            _ => Err(format!("Unknown locale: {}", s)),
        }
    }
}

impl Locale {
    /// Returns display label for the locale.
    pub fn label(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Zh => "中文",
        }
    }

    /// Returns all supported locales.
    pub fn all() -> Vec<Locale> {
        vec![Locale::En, Locale::Zh]
    }
}

/// Get translation for the given key using current locale from context.
///
/// Must be called within a Leptos reactive context where `ReadSignal<Locale>`
/// has been provided.
pub fn t(key: &str) -> String {
    let locale = use_context::<ReadSignal<Locale>>()
        .map(|s| s.get())
        .unwrap_or(Locale::En);
    translate(locale, key)
}

/// Core translation lookup.
fn translate(locale: Locale, key: &str) -> String {
    let result = match locale {
        Locale::En => en(key),
        Locale::Zh => zh(key),
    };
    result.to_string()
}

fn en(key: &str) -> &'static str {
    match key {
        // App
        "app.title" => "Alias Manager",
        "page.not_found" => "Page not found",
        // Navigation
        "nav.aliases" => "Aliases",
        "nav.templates" => "Templates",
        "nav.settings" => "Settings",
        // Theme
        "theme.light" => "Light Mode",
        "theme.dark" => "Dark Mode",
        // Search
        "search.placeholder" => "Search aliases...",
        // Alias page
        "alias.import" => "Import",
        "alias.export" => "Export",
        "alias.add_btn" => "+ Add Alias",
        "alias.import_prompt" => "Paste alias JSON data:",
        "alias.import_partial" => "Imported {} aliases, failed: {}",
        "alias.json_parse_error" => "JSON parse failed: {}",
        "alias.delete_partial" => "Some deletions failed: {}",
        // Alias list
        "alias.count" => "{} aliases total",
        "alias.count_filtered" => "Found {} / {} aliases",
        "alias.delete_selected" => "Delete Selected ({})",
        "alias.select_all" => "Select All",
        "alias.empty_title" => "No aliases yet",
        "alias.empty_desc" => "Click the \"+ Add Alias\" button to create your first alias",
        "alias.search_empty_title" => "No matching aliases",
        "alias.search_empty_desc" => "No aliases matching \"{}\"",
        "alias.edit" => "Edit",
        "alias.delete" => "Delete",
        // Alias form
        "form.edit_title" => "Edit Alias",
        "form.add_title" => "Add Alias",
        "form.name_label" => "Alias Name",
        "form.name_placeholder" => "e.g. gs",
        "form.name_hint" => "Only letters, numbers, underscores, and hyphens",
        "form.command_label" => "Command",
        "form.command_placeholder" => "e.g. git status",
        "form.tags_label" => "Tags",
        "form.tags_placeholder" => "e.g. git, shortcut (comma separated)",
        "form.tags_hint" => "Separate multiple tags with commas",
        "form.cancel" => "Cancel",
        "form.save" => "Save",
        "form.add" => "Add",
        // Validation
        "validate.name_empty" => "Alias name cannot be empty",
        "validate.name_hyphen" => "Alias name cannot start with a hyphen",
        "validate.name_chars" => "Alias name can only contain letters, numbers, underscores, and hyphens",
        "validate.command_empty" => "Command cannot be empty",
        // Template page
        "template.title" => "Templates",
        "template.import_selected" => "Import Selected",
        "template.import_success" => "Successfully imported {} aliases",
        "template.all" => "All",
        "template.empty_title" => "No Templates",
        "template.empty_desc" => "No templates available in this category",
        // Template categories
        "category.file_ops" => "File Ops",
        "category.network" => "Network",
        "category.custom" => "Custom",
        // Settings page
        "settings.title" => "Settings",
        "settings.language" => "Language",
        "settings.language_desc" => "Choose the display language",
        "settings.shell_config" => "Shell Configuration",
        "settings.shell_type" => "Shell Type",
        "settings.shell_type_desc" => "Select the shell config file to manage",
        "settings.custom_path" => "Custom Config Path",
        "settings.custom_path_desc" => "Leave empty to use default path",
        "settings.custom_path_placeholder" => "e.g. /home/user/.custom_bashrc",
        "settings.save" => "Save",
        "settings.data_management" => "Data Management",
        "settings.auto_refresh" => "Auto Refresh",
        "settings.auto_refresh_desc" => "Auto-refresh alias list when config file changes",
        "settings.manual_refresh" => "Manual Refresh",
        "settings.manual_refresh_desc" => "Re-read config file now",
        "settings.refresh_btn" => "Refresh",
        "settings.about" => "About",
        "settings.about_desc" => "A Shell alias manager built with Tauri v2 + Leptos 0.8",
        "settings.about_support" => "Supports Bash, Zsh, Fish config file management",
        "settings.shell_updated" => "Shell type updated",
        "settings.path_updated" => "Config path updated",
        "settings.language_updated" => "Language updated",
        _ => "[missing]",
    }
}

fn zh(key: &str) -> &'static str {
    match key {
        // App
        "app.title" => "别名管理器",
        "page.not_found" => "页面未找到",
        // Navigation
        "nav.aliases" => "别名管理",
        "nav.templates" => "模板库",
        "nav.settings" => "设置",
        // Theme
        "theme.light" => "浅色模式",
        "theme.dark" => "暗色模式",
        // Search
        "search.placeholder" => "搜索别名...",
        // Alias page
        "alias.import" => "导入",
        "alias.export" => "导出",
        "alias.add_btn" => "+ 添加别名",
        "alias.import_prompt" => "请粘贴别名 JSON 数据:",
        "alias.import_partial" => "导入了 {} 个别名，失败: {}",
        "alias.json_parse_error" => "JSON 解析失败: {}",
        "alias.delete_partial" => "部分删除失败: {}",
        // Alias list
        "alias.count" => "共 {} 个别名",
        "alias.count_filtered" => "找到 {} / {} 个别名",
        "alias.delete_selected" => "删除选中 ({})",
        "alias.select_all" => "全选",
        "alias.empty_title" => "还没有别名",
        "alias.empty_desc" => "点击右上角的「添加别名」按钮创建你的第一个别名",
        "alias.search_empty_title" => "没有找到匹配的别名",
        "alias.search_empty_desc" => "没有与「{}」匹配的别名",
        "alias.edit" => "编辑",
        "alias.delete" => "删除",
        // Alias form
        "form.edit_title" => "编辑别名",
        "form.add_title" => "添加别名",
        "form.name_label" => "别名名称",
        "form.name_placeholder" => "例如: gs",
        "form.name_hint" => "只能包含字母、数字、下划线和连字符",
        "form.command_label" => "命令",
        "form.command_placeholder" => "例如: git status",
        "form.tags_label" => "标签",
        "form.tags_placeholder" => "例如: git, 快捷命令（逗号分隔）",
        "form.tags_hint" => "用逗号分隔多个标签",
        "form.cancel" => "取消",
        "form.save" => "保存",
        "form.add" => "添加",
        // Validation
        "validate.name_empty" => "别名名称不能为空",
        "validate.name_hyphen" => "别名名称不能以连字符开头",
        "validate.name_chars" => "别名名称只能包含字母、数字、下划线和连字符",
        "validate.command_empty" => "命令不能为空",
        // Template page
        "template.title" => "模板库",
        "template.import_selected" => "导入选中",
        "template.import_success" => "成功导入 {} 个别名",
        "template.all" => "全部",
        "template.empty_title" => "没有模板",
        "template.empty_desc" => "该分类下没有可用的模板",
        // Template categories
        "category.file_ops" => "文件操作",
        "category.network" => "网络",
        "category.custom" => "自定义",
        // Settings page
        "settings.title" => "设置",
        "settings.language" => "语言",
        "settings.language_desc" => "选择界面显示语言",
        "settings.shell_config" => "Shell 配置",
        "settings.shell_type" => "Shell 类型",
        "settings.shell_type_desc" => "选择你要管理的 Shell 配置文件",
        "settings.custom_path" => "自定义配置路径",
        "settings.custom_path_desc" => "留空则使用默认路径",
        "settings.custom_path_placeholder" => "例如: /home/user/.custom_bashrc",
        "settings.save" => "保存",
        "settings.data_management" => "数据管理",
        "settings.auto_refresh" => "自动刷新",
        "settings.auto_refresh_desc" => "配置文件变更时自动刷新别名列表",
        "settings.manual_refresh" => "手动刷新",
        "settings.manual_refresh_desc" => "立即重新读取配置文件",
        "settings.refresh_btn" => "刷新",
        "settings.about" => "关于",
        "settings.about_desc" => "基于 Tauri v2 + Leptos 0.8 构建的 Shell 别名管理器",
        "settings.about_support" => "支持 Bash、Zsh、Fish 配置文件管理",
        "settings.shell_updated" => "Shell 类型已更新",
        "settings.path_updated" => "配置路径已更新",
        "settings.language_updated" => "语言已更新",
        _ => en(key),
    }
}

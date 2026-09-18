/// Internationalization (i18n) module.
///
/// Provides locale management and translation lookup via Leptos context.
/// Supports English (default) and Chinese.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Locale {
    /// English (default).
    #[default]
    En,
    /// Chinese (Simplified).
    Zh,
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
        "alias.import_file" => "Import from File",
        "alias.import_partial" => "Imported {} aliases, failed: {}",
        "alias.import_preview_title" => "Import Preview",
        "alias.import_preview_summary" => "New: {} · Overwritable: {} · Unchanged: {} · Invalid: {}",
        "alias.import_status_new" => "New",
        "alias.import_status_conflict" => "Exists",
        "alias.import_status_unchanged" => "Unchanged",
        "alias.import_status_invalid" => "Invalid",
        "alias.import_overwrite_col" => "Overwrite",
        "alias.import_confirm" => "Import",
        "alias.import_dedup_note" => "{} duplicate entries in the file were merged (last occurrence wins)",
        "alias.import_overwrite_all" => "Overwrite All",
        "alias.view_config" => "View Config",
        "alias.conflict_title" => "Alias Conflict",
        "alias.conflict_msg" => "Alias \"{}\" already exists. Overwrite it?",
        "alias.conflict_overwrite" => "Overwrite",
        "alias.conflict_keep" => "Keep Existing",
        "alias.json_parse_error" => "JSON parse failed: {}",
        "alias.delete_partial" => "Some deletions failed: {}",
        "alias.delete_success" => "Deleted {} aliases",
        "alias.import_success" => "Imported {} aliases",
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
        "settings.instant_apply" => "Instant Apply",
        "settings.instant_apply_desc" => "Run `source` on the config file after changes. Only affects terminals opened after or started by this app.",
        "template.import_skipped" => "Imported {} new, skipped {} existing",
        "template.custom_new" => "Create custom template",
        "template.custom_name" => "Name",
        "template.custom_command" => "Command",
        "template.custom_desc" => "Description",
        "template.custom_save" => "Save",
        "template.custom_required" => "Name and command are required",
        "template.delete" => "Delete",
        "tag.filter_all" => "All tags",
        "tag.click_to_filter" => "Click to filter by this tag",
        "settings.manual_refresh" => "Manual Refresh",
        "settings.manual_refresh_desc" => "Re-read config file now",
        "settings.refresh_btn" => "Refresh",
        "settings.about" => "About",
        "settings.about_desc" => "A Shell alias manager built with Tauri v2 + Leptos 0.8",
        "settings.about_support" => "Supports Bash, Zsh, Fish config file management",
        "settings.shell_updated" => "Shell type updated",
        "settings.path_updated" => "Config path updated",
        "settings.language_updated" => "Language updated",
        // Update check
        "update.check_btn" => "Check for Updates",
        "update.checking" => "Checking...",
        "update.latest_version" => "Latest version",
        "update.download_latest" => "Download Latest",
        "update.up_to_date" => "You're up to date",
        "update.network_error" => "Failed to check for updates. Please check your network connection.",
        // Error codes (backend AppError localized by code)
        "error.io_error" => "I/O error: {}",
        "error.parse_error" => "Failed to parse config file: {}",
        "error.config_not_found" => "Config file not found: {}",
        "error.alias_exists" => "Alias \"{}\" already exists",
        "error.alias_conflict" => "Alias \"{}\" already exists",
        "error.alias_not_found" => "Alias \"{}\" not found",
        "error.invalid_alias_name" => "Invalid alias name: {}",
        "error.json_error" => "JSON error: {}",
        "error.network_error" => "Network error: {}",
        "error.rate_limited" => "GitHub API rate limited, please try again later",
        "error.release_not_found" => "No releases found for this repository",
        // Backups
        "settings.backups" => "Backups",
        "settings.backups_desc" => "A backup is created automatically before every change. Restoring a backup overwrites the current config file.",
        "settings.backup_time" => "Time",
        "settings.backup_path" => "File",
        "settings.backup_size" => "Size",
        "settings.backup_restore" => "Restore",
        "settings.backup_restore_confirm" => "Restore this backup? The current config file will be overwritten.",
        "settings.backup_restored" => "Backup restored",
        "settings.backup_empty" => "No backups yet",
        "settings.backup_refresh" => "Refresh",
        // Config viewer
        "config_viewer.title" => "Config File",
        "config_viewer.loading" => "Loading...",
        "config_viewer.empty" => "Config file is empty",
        "config_viewer.close" => "Close",
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
        "alias.import_file" => "从文件导入",
        "alias.import_partial" => "导入了 {} 个别名，失败: {}",
        "alias.import_preview_title" => "导入预览",
        "alias.import_preview_summary" => "新增：{} · 可覆盖：{} · 无变化：{} · 无效：{}",
        "alias.import_status_new" => "新增",
        "alias.import_status_conflict" => "已存在",
        "alias.import_status_unchanged" => "无变化",
        "alias.import_status_invalid" => "无效",
        "alias.import_overwrite_col" => "覆盖",
        "alias.import_confirm" => "导入",
        "alias.import_dedup_note" => "文件中 {} 条重复条目已合并（以最后一条为准）",
        "alias.import_overwrite_all" => "全部覆盖",
        "alias.view_config" => "查看配置",
        "alias.conflict_title" => "别名冲突",
        "alias.conflict_msg" => "别名「{}」已存在，是否覆盖？",
        "alias.conflict_overwrite" => "覆盖",
        "alias.conflict_keep" => "保留原值",
        "alias.json_parse_error" => "JSON 解析失败: {}",
        "alias.delete_partial" => "部分删除失败: {}",
        "alias.delete_success" => "已删除 {} 个别名",
        "alias.import_success" => "已导入 {} 个别名",
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
        "settings.instant_apply" => "即时生效",
        "settings.instant_apply_desc" => {
            "变更后自动对配置文件执行 source。仅对之后新开或由本应用启动的终端生效。"
        },
        "template.import_skipped" => "新增 {} 个，跳过已存在 {} 个",
        "template.custom_new" => "新建自定义模板",
        "template.custom_name" => "名称",
        "template.custom_command" => "命令",
        "template.custom_desc" => "描述",
        "template.custom_save" => "保存",
        "template.custom_required" => "名称和命令不能为空",
        "template.delete" => "删除",
        "tag.filter_all" => "全部标签",
        "tag.click_to_filter" => "点击按此标签筛选",
        "settings.manual_refresh" => "手动刷新",
        "settings.manual_refresh_desc" => "立即重新读取配置文件",
        "settings.refresh_btn" => "刷新",
        "settings.about" => "关于",
        "settings.about_desc" => "基于 Tauri v2 + Leptos 0.8 构建的 Shell 别名管理器",
        "settings.about_support" => "支持 Bash、Zsh、Fish 配置文件管理",
        "settings.shell_updated" => "Shell 类型已更新",
        "settings.path_updated" => "配置路径已更新",
        "settings.language_updated" => "语言已更新",
        // 更新检查
        "update.check_btn" => "检查更新",
        "update.checking" => "检查中...",
        "update.latest_version" => "最新版本",
        "update.download_latest" => "下载最新版本",
        "update.up_to_date" => "当前已是最新版本",
        "update.network_error" => "检查更新失败，请检查网络连接。",
        // 错误码（按后端错误码本地化）
        "error.io_error" => "I/O 错误：{}",
        "error.parse_error" => "配置文件解析失败：{}",
        "error.config_not_found" => "配置文件未找到：{}",
        "error.alias_exists" => "别名「{}」已存在",
        "error.alias_conflict" => "别名「{}」已存在",
        "error.alias_not_found" => "别名「{}」未找到",
        "error.invalid_alias_name" => "非法别名名称：{}",
        "error.json_error" => "JSON 错误：{}",
        "error.network_error" => "网络错误：{}",
        "error.rate_limited" => "请求过于频繁，请稍后再试",
        "error.release_not_found" => "未找到仓库的发布版本",
        // 备份管理
        "settings.backups" => "备份管理",
        "settings.backups_desc" => {
            "每次修改配置文件前都会自动创建备份，恢复备份将覆盖当前配置文件。"
        },
        "settings.backup_time" => "时间",
        "settings.backup_path" => "文件",
        "settings.backup_size" => "大小",
        "settings.backup_restore" => "恢复",
        "settings.backup_restore_confirm" => "确定恢复该备份？当前配置文件将被覆盖。",
        "settings.backup_restored" => "已恢复备份",
        "settings.backup_empty" => "暂无备份",
        "settings.backup_refresh" => "刷新",
        // 配置文件查看
        "config_viewer.title" => "配置文件",
        "config_viewer.loading" => "加载中...",
        "config_viewer.empty" => "配置文件为空",
        "config_viewer.close" => "关闭",
        _ => en(key),
    }
}

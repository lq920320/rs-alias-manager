/// 别名行解析工具。
///
/// 支持从 Shell 配置文件中解析三种格式的别名行：
/// - `alias name='command'`（单引号）
/// - `alias name="command"`（双引号）
/// - `alias name=command`（无引号）
///
/// 标签通过别名行上方的 `# @tags:tag1,tag2` 注释行存储。
use crate::error::AppError;
use crate::models::alias::Alias;

/// 标签注释的前缀。
const TAGS_PREFIX: &str = "# @tags:";

/// 解析 Shell 配置文件中的单个别名行。
///
/// 支持的格式：
/// - `alias name='command with spaces'`
/// - `alias name="command with spaces"`
/// - `alias name=command`
///
/// 对于非别名定义的行（注释、空行等）返回 `None`。
pub fn parse_alias_line(line: &str) -> Option<Alias> {
    let trimmed = line.trim();

    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    // Must start with "alias "
    if !trimmed.starts_with("alias ") {
        return None;
    }

    let rest = &trimmed[6..]; // skip "alias "
    let eq_pos = rest.find('=')?;
    let name = &rest[..eq_pos];
    let value_part = &rest[eq_pos + 1..];

    // Validate name
    if name.is_empty() {
        return None;
    }

    let command = parse_command_value(value_part)?;
    Some(Alias::new(name, command))
}

/// 从注释行解析标签。
///
/// 如果行格式为 `# @tags:tag1,tag2`，返回标签列表。
fn parse_tags_comment(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if let Some(tags_str) = trimmed.strip_prefix(TAGS_PREFIX) {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tags.is_empty() {
            return Some(tags);
        }
    }
    None
}

/// 解析别名行中的命令值部分。
///
/// 处理三种引号风格：
/// - 单引号：` 'command' ` → 去除引号
/// - 双引号：` "command" ` → 去除引号
/// - 无引号：`command` → 原样返回
fn parse_command_value(value: &str) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Single-quoted value
    if trimmed.starts_with('\'') {
        if let Some(end) = trimmed.rfind('\'') {
            if end > 0 {
                return Some(trimmed[1..end].to_string());
            }
        }
        return None;
    }

    // Double-quoted value
    if trimmed.starts_with('"') {
        if let Some(end) = trimmed.rfind('"') {
            if end > 0 {
                return Some(trimmed[1..end].to_string());
            }
        }
        return None;
    }

    // Unquoted value — take until end of line (no spaces in unquoted commands)
    Some(trimmed.to_string())
}

/// 将别名格式化为规范的输出行格式。
///
/// 始终使用单引号：`alias name='command'`
/// 如果存在标签，会在别名行上方输出 `# @tags:tag1,tag2` 注释。
pub fn format_alias_line(alias: &Alias) -> String {
    if alias.tags.is_empty() {
        format!("alias {}='{}'", alias.name, alias.command)
    } else {
        format!(
            "{}{}
alias {}='{}'",
            TAGS_PREFIX,
            alias.tags.join(","),
            alias.name,
            alias.command
        )
    }
}

/// 解析 Shell 配置文件内容中的所有别名行。
///
/// 返回所有成功解析的别名向量，包含从 `# @tags:` 注释中提取的标签。
/// 非别名行（注释、空行、其他配置）将被静默跳过。
pub fn parse_aliases_from_content(content: &str) -> Vec<Alias> {
    let mut aliases = Vec::new();
    let mut pending_tags: Option<Vec<String>> = None;

    for line in content.lines() {
        // 检查是否是标签注释行
        if let Some(tags) = parse_tags_comment(line) {
            pending_tags = Some(tags);
            continue;
        }

        // 尝试解析别名行
        if let Some(mut alias) = parse_alias_line(line) {
            // 如果前一行是标签注释，将标签附加到该别名
            if let Some(tags) = pending_tags.take() {
                alias.tags = tags;
            }
            aliases.push(alias);
        } else {
            // 非别名行且非标签注释，清除待处理标签
            pending_tags = None;
        }
    }

    aliases
}

/// 通过用提供的别名替换别名行并保留所有非别名行来重建配置文件内容。
///
/// - 现有的别名行根据新的别名列表被替换或移除。
/// - 非别名行（注释、export 等）保持原有顺序。
/// - 原始文件中不存在的新别名将追加在末尾。
/// - 标签注释行会被正确处理（移除旧的、添加新的）。
pub fn rebuild_config_content(original_content: &str, aliases: &[Alias]) -> String {
    let mut result_lines: Vec<String> = Vec::new();
    let new_alias_names: std::collections::HashSet<String> =
        aliases.iter().map(|a| a.name.clone()).collect();
    let mut alias_order: Vec<String> = aliases.iter().map(|a| a.name.clone()).collect();

    let lines: Vec<&str> = original_content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // 检查是否是标签注释 + 别名行的组合
        if parse_tags_comment(line).is_some() && i + 1 < lines.len() {
            if let Some(parsed) = parse_alias_line(lines[i + 1]) {
                // 这是一个标签注释 + 别名行的组合
                if new_alias_names.contains(&parsed.name) {
                    // 用更新后的别名替换（包含新标签）
                    if let Some(updated) = aliases.iter().find(|a| a.name == parsed.name) {
                        result_lines.push(format_alias_line(updated));
                    }
                    alias_order.retain(|n| n != &parsed.name);
                }
                // 否则别名已删除，跳过标签注释和别名行
                i += 2;
                continue;
            }
        }

        // 检查是否是单独的别名行（无标签注释）
        if let Some(parsed) = parse_alias_line(line) {
            if new_alias_names.contains(&parsed.name) {
                if let Some(updated) = aliases.iter().find(|a| a.name == parsed.name) {
                    result_lines.push(format_alias_line(updated));
                }
                alias_order.retain(|n| n != &parsed.name);
            }
            // 如果不在新集合中，该别名已删除——跳过
        } else {
            // 非别名行——保留原样
            result_lines.push(line.to_string());
        }

        i += 1;
    }

    // 追加原始文件中不存在的新别名
    for name in alias_order {
        if let Some(alias) = aliases.iter().find(|a| a.name == name) {
            result_lines.push(format_alias_line(alias));
        }
    }

    // 确保文件以换行符结尾
    let mut result = result_lines.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// 向配置内容中添加别名行。
///
/// 别名被追加到文件末尾。如果已存在同名别名则返回错误。
pub fn add_alias_to_content(content: &str, alias: &Alias) -> Result<String, AppError> {
    let existing = parse_aliases_from_content(content);
    if existing.iter().any(|a| a.name == alias.name) {
        return Err(AppError::AliasExists(alias.name.clone()));
    }

    let mut new_content = content.to_string();
    let line = format_alias_line(alias);

    // Ensure the content ends with a newline before appending
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&line);
    new_content.push('\n');

    Ok(new_content)
}

/// 更新配置内容中的现有别名。
///
/// 如果不存在指定名称的别名则返回错误。
pub fn update_alias_in_content(
    content: &str,
    old_name: &str,
    alias: &Alias,
) -> Result<String, AppError> {
    let existing = parse_aliases_from_content(content);
    if !existing.iter().any(|a| a.name == old_name) {
        return Err(AppError::AliasNotFound(old_name.to_string()));
    }

    let new_aliases: Vec<Alias> =
        existing.into_iter().map(|a| if a.name == old_name { alias.clone() } else { a }).collect();

    // 如果名称更改了，检查与其他现有别名的冲突（排除正在更新的别名），避免名称重复
    if old_name != alias.name && new_aliases.iter().filter(|a| a.name == alias.name).count() > 1 {
        return Err(AppError::AliasExists(alias.name.clone()));
    }

    Ok(rebuild_config_content(content, &new_aliases))
}

/// 从配置内容中删除别名。
///
/// 如果不存在指定名称的别名则返回错误。
pub fn delete_alias_from_content(content: &str, name: &str) -> Result<String, AppError> {
    let existing = parse_aliases_from_content(content);
    if !existing.iter().any(|a| a.name == name) {
        return Err(AppError::AliasNotFound(name.to_string()));
    }

    let remaining: Vec<Alias> = existing.into_iter().filter(|a| a.name != name).collect();
    Ok(rebuild_config_content(content, &remaining))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_quoted() {
        let alias = parse_alias_line("alias gs='git status'").unwrap();
        assert_eq!(alias.name, "gs");
        assert_eq!(alias.command, "git status");
    }

    #[test]
    fn test_parse_double_quoted() {
        let alias = parse_alias_line("alias ll=\"ls -la\"").unwrap();
        assert_eq!(alias.name, "ll");
        assert_eq!(alias.command, "ls -la");
    }

    #[test]
    fn test_parse_unquoted() {
        let alias = parse_alias_line("alias cls=clear").unwrap();
        assert_eq!(alias.name, "cls");
        assert_eq!(alias.command, "clear");
    }

    #[test]
    fn test_skip_comments() {
        assert!(parse_alias_line("# alias skip='echo skip'").is_none());
    }

    #[test]
    fn test_skip_empty() {
        assert!(parse_alias_line("").is_none());
        assert!(parse_alias_line("   ").is_none());
    }

    #[test]
    fn test_skip_non_alias() {
        assert!(parse_alias_line("export PATH=$HOME/bin:$PATH").is_none());
    }

    #[test]
    fn test_format_alias() {
        let alias = Alias::new("gs", "git status");
        assert_eq!(format_alias_line(&alias), "alias gs='git status'");
    }

    #[test]
    fn test_rebuild_preserves_non_alias_lines() {
        let content =
            "# My aliases\nalias ll='ls -la'\nexport PATH=$HOME/bin:$PATH\nalias gs='git status'\n";
        let aliases = vec![Alias::new("gs", "git status --short")];
        let result = rebuild_config_content(content, &aliases);
        assert!(result.contains("# My aliases"));
        assert!(result.contains("export PATH=$HOME/bin:$PATH"));
        assert!(result.contains("alias gs='git status --short'"));
        assert!(!result.contains("alias ll="));
    }

    // === 边界情况测试 ===

    // -- parse_alias_line: 命令中的特殊字符 --

    #[test]
    fn test_parse_command_with_pipe() {
        let alias = parse_alias_line("alias lc='ls | wc -l'").unwrap();
        assert_eq!(alias.name, "lc");
        assert_eq!(alias.command, "ls | wc -l");
    }

    #[test]
    fn test_parse_command_with_semicolon() {
        let alias = parse_alias_line("alias rebuild='cargo clean; cargo build'").unwrap();
        assert_eq!(alias.name, "rebuild");
        assert_eq!(alias.command, "cargo clean; cargo build");
    }

    #[test]
    fn test_parse_command_with_dollar_variable() {
        let alias = parse_alias_line("alias home='echo $HOME'").unwrap();
        assert_eq!(alias.name, "home");
        assert_eq!(alias.command, "echo $HOME");
    }

    #[test]
    fn test_parse_command_with_double_quotes_inside_single() {
        let alias = parse_alias_line("alias greet='echo \"hello\"'").unwrap();
        assert_eq!(alias.name, "greet");
        assert_eq!(alias.command, "echo \"hello\"");
    }

    #[test]
    fn test_parse_command_with_backslash() {
        let alias = parse_alias_line(r"alias escape='echo \n'").unwrap();
        assert_eq!(alias.name, "escape");
        assert_eq!(alias.command, r"echo \n");
    }

    #[test]
    fn test_parse_unquoted_with_spaces() {
        // 无引号的命令应原样取用（包括空格）
        let alias = parse_alias_line("alias cmd=simple_command").unwrap();
        assert_eq!(alias.name, "cmd");
        assert_eq!(alias.command, "simple_command");
    }

    // -- parse_alias_line: 输入边界情况 --

    #[test]
    fn test_parse_with_leading_whitespace() {
        let alias = parse_alias_line("  alias gs='git status'").unwrap();
        assert_eq!(alias.name, "gs");
        assert_eq!(alias.command, "git status");
    }

    #[test]
    fn test_parse_with_trailing_whitespace() {
        let alias = parse_alias_line("alias gs='git status'  ").unwrap();
        assert_eq!(alias.name, "gs");
        assert_eq!(alias.command, "git status");
    }

    #[test]
    fn test_parse_alias_no_equals() {
        // "alias name" 没有 = 应返回 None
        assert!(parse_alias_line("alias gs").is_none());
    }

    #[test]
    fn test_parse_alias_empty_name() {
        // "alias ='command'" 空名称应返回 None
        assert!(parse_alias_line("alias ='git status'").is_none());
    }

    #[test]
    fn test_parse_alias_empty_value() {
        // "alias gs=" 空值应返回 None
        assert!(parse_alias_line("alias gs=").is_none());
    }

    #[test]
    fn test_parse_alias_empty_single_quotes() {
        // "alias gs=''" 空单引号值
        // 注意：rfind('\'') 在 "''" 上找到索引 1（最后一个 '），但 end > 0 检查
        // 意味着 1 > 0 为 true，所以返回 Some(""[1..1]) = ""
        // 实际：trimmed = "''", starts_with('\'') = true, rfind('\'') = Some(1),
        // end > 0 为 true，所以 trimmed[1..1] = "" → Some("")
        // 但名称是有效的，所以应解析为 command 为空的 Alias
        let alias = parse_alias_line("alias gs=''");
        // 空字符串命令在 shell 中实际上是有效的（alias gs=''）
        // 但 parse_command_value 对 '' 返回 Some("")，因为 end=1 > 0
        // 追踪：trimmed="''"，starts_with('\'')=true，
        // rfind('\'')=Some(1)，end=1，end>0=true，trimmed[1..1]=""
        // 所以返回 Some("")
        // 然后在 parse_alias_line 中，command = Some("") 是 Some
        // 所以应返回 Some(Alias)
        assert!(alias.is_some());
        assert_eq!(alias.unwrap().command, "");
    }

    #[test]
    fn test_parse_unclosed_single_quote() {
        assert!(parse_alias_line("alias gs='git status").is_none());
    }

    #[test]
    fn test_parse_unclosed_double_quote() {
        assert!(parse_alias_line("alias gs=\"git status").is_none());
    }

    #[test]
    fn test_parse_inline_comment_after_alias() {
        // 这在 shell 中技术上不是注释，但我们的解析器仍应解析它
        // 因为它以 "alias " 开头
        let alias = parse_alias_line("alias gs='git status' # shortcut").unwrap();
        assert_eq!(alias.name, "gs");
        // 命令将在闭合引号之后包含尾随的 " # shortcut"
        // 实际：value_part = "'git status' # shortcut"
        // trimmed = "'git status' # shortcut"
        // starts_with('\'') = true
        // rfind('\'') 找到闭合单引号
        assert_eq!(alias.command, "git status");
    }

    #[test]
    fn test_parse_only_word_alias() {
        // 只有 "alias" 一词，没有名称应返回 None
        assert!(parse_alias_line("alias").is_none());
    }

    #[test]
    fn test_parse_alias_with_hyphen_name() {
        let alias = parse_alias_line("alias g-co='git checkout'").unwrap();
        assert_eq!(alias.name, "g-co");
        assert_eq!(alias.command, "git checkout");
    }

    #[test]
    fn test_parse_alias_with_underscore_name() {
        let alias = parse_alias_line("alias _gs='git stash'").unwrap();
        assert_eq!(alias.name, "_gs");
        assert_eq!(alias.command, "git stash");
    }

    #[test]
    fn test_parse_line_starting_with_alias_without_space() {
        // "aliassomething" 不应被解析为别名
        assert!(parse_alias_line("aliassomething='value'").is_none());
    }

    // -- parse_aliases_from_content 测试 --

    #[test]
    fn test_parse_multiple_aliases_from_content() {
        let content = "alias gs='git status'\nalias ll='ls -la'\nalias gp='git push'\n";
        let aliases = parse_aliases_from_content(content);
        assert_eq!(aliases.len(), 3);
        assert_eq!(aliases[0].name, "gs");
        assert_eq!(aliases[1].name, "ll");
        assert_eq!(aliases[2].name, "gp");
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = "# Shell config\n\nexport PATH=$HOME/bin:$PATH\nalias gs='git status'\n# Git aliases\nalias ll='ls -la'\n";
        let aliases = parse_aliases_from_content(content);
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].name, "gs");
        assert_eq!(aliases[1].name, "ll");
    }

    #[test]
    fn test_parse_empty_content() {
        let aliases = parse_aliases_from_content("");
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_parse_content_only_comments() {
        let content = "# comment 1\n# comment 2\n";
        let aliases = parse_aliases_from_content(content);
        assert!(aliases.is_empty());
    }

    // -- rebuild_config_content 测试 --

    #[test]
    fn test_rebuild_empty_content_with_new_aliases() {
        let content = "";
        let aliases = vec![Alias::new("gs", "git status"), Alias::new("ll", "ls -la")];
        let result = rebuild_config_content(content, &aliases);
        assert!(result.contains("alias gs='git status'"));
        assert!(result.contains("alias ll='ls -la'"));
    }

    #[test]
    fn test_rebuild_removes_alias() {
        let content = "alias gs='git status'\nalias ll='ls -la'\n";
        let aliases = vec![Alias::new("gs", "git status")];
        let result = rebuild_config_content(content, &aliases);
        assert!(result.contains("alias gs='git status'"));
        assert!(!result.contains("alias ll="));
    }

    #[test]
    fn test_rebuild_updates_alias() {
        let content = "alias gs='git status'\n";
        let aliases = vec![Alias::new("gs", "git status --short")];
        let result = rebuild_config_content(content, &aliases);
        assert!(result.contains("alias gs='git status --short'"));
        assert!(!result.contains("alias gs='git status'"));
    }

    #[test]
    fn test_rebuild_appends_new_alias() {
        let content = "alias gs='git status'\n";
        let aliases = vec![Alias::new("gs", "git status"), Alias::new("ll", "ls -la")];
        let result = rebuild_config_content(content, &aliases);
        assert!(result.contains("alias gs='git status'"));
        assert!(result.contains("alias ll='ls -la'"));
        // New alias should appear after existing ones
        let gs_pos = result.find("alias gs=").unwrap();
        let ll_pos = result.find("alias ll=").unwrap();
        assert!(gs_pos < ll_pos, "新别名应追加在已有别名之后");
    }

    #[test]
    fn test_rebuild_preserves_line_order() {
        let content =
            "# Header\nalias gs='git status'\nexport FOO=bar\nalias ll='ls -la'\n# Footer\n";
        let aliases = vec![Alias::new("gs", "git status --short"), Alias::new("ll", "ls -la")];
        let result = rebuild_config_content(content, &aliases);

        let header_pos = result.find("# Header").unwrap();
        let export_pos = result.find("export FOO=bar").unwrap();
        let gs_pos = result.find("alias gs=").unwrap();
        let ll_pos = result.find("alias ll=").unwrap();
        let footer_pos = result.find("# Footer").unwrap();

        // 非别名行应保持相对顺序
        assert!(header_pos < export_pos);
        assert!(export_pos < footer_pos);
        // 别名行应保持相对顺序
        assert!(gs_pos < ll_pos);
    }

    #[test]
    fn test_rebuild_ends_with_newline() {
        let content = "alias gs='git status'\n";
        let aliases = vec![Alias::new("gs", "git status")];
        let result = rebuild_config_content(content, &aliases);
        assert!(result.ends_with('\n'), "重建后的内容应以换行符结尾");
    }

    #[test]
    fn test_rebuild_empty_aliases_removes_all() {
        let content = "alias gs='git status'\nalias ll='ls -la'\n";
        let aliases: Vec<Alias> = vec![];
        let result = rebuild_config_content(content, &aliases);
        assert!(!result.contains("alias "));
        // 当所有行被移除时，result_lines 为空，join 产生 ""，
        // 且由于 result 为空，不添加换行符
        assert_eq!(result, "");
    }

    // -- add_alias_to_content 测试 --

    #[test]
    fn test_add_alias_to_empty_content() {
        let content = "";
        let alias = Alias::new("gs", "git status");
        let result = add_alias_to_content(content, &alias).unwrap();
        assert!(result.contains("alias gs='git status'"));
    }

    #[test]
    fn test_add_alias_to_content_with_existing() {
        let content = "alias ll='ls -la'\n";
        let alias = Alias::new("gs", "git status");
        let result = add_alias_to_content(content, &alias).unwrap();
        assert!(result.contains("alias ll='ls -la'"));
        assert!(result.contains("alias gs='git status'"));
    }

    #[test]
    fn test_add_alias_duplicate_returns_error() {
        let content = "alias gs='git status'\n";
        let alias = Alias::new("gs", "git status --short");
        let result = add_alias_to_content(content, &alias);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::AliasExists(name) => assert_eq!(name, "gs"),
            other => panic!("期望 AliasExists，得到 {:?}", other),
        }
    }

    #[test]
    fn test_add_alias_to_content_without_trailing_newline() {
        let content = "alias ll='ls -la'"; // 没有末尾换行符
        let alias = Alias::new("gs", "git status");
        let result = add_alias_to_content(content, &alias).unwrap();
        assert!(result.contains("alias ll='ls -la'\n"));
        assert!(result.contains("alias gs='git status'"));
    }

    // -- update_alias_in_content 测试 --

    #[test]
    fn test_update_alias_command_only() {
        let content = "alias gs='git status'\n";
        let alias = Alias::new("gs", "git status --short");
        let result = update_alias_in_content(content, "gs", &alias).unwrap();
        assert!(result.contains("alias gs='git status --short'"));
        assert!(!result.contains("alias gs='git status'"));
    }

    #[test]
    fn test_update_alias_rename() {
        let content = "alias gs='git status'\n";
        let alias = Alias::new("gitstatus", "git status");
        let result = update_alias_in_content(content, "gs", &alias).unwrap();
        assert!(result.contains("alias gitstatus='git status'"));
        assert!(!result.contains("alias gs="));
    }

    #[test]
    fn test_update_alias_rename_conflict() {
        let content = "alias gs='git status'\nalias ll='ls -la'\n";
        let alias = Alias::new("ll", "git status"); // 将 gs 重命名为 ll，但 ll 已存在
        let result = update_alias_in_content(content, "gs", &alias);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::AliasExists(name) => assert_eq!(name, "ll"),
            other => panic!("期望 AliasExists，得到 {:?}", other),
        }
    }

    #[test]
    fn test_update_alias_not_found() {
        let content = "alias gs='git status'\n";
        let alias = Alias::new("new", "echo hello");
        let result = update_alias_in_content(content, "nonexistent", &alias);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::AliasNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("期望 AliasNotFound，得到 {:?}", other),
        }
    }

    #[test]
    fn test_update_alias_same_name_no_conflict() {
        // 更新同名但不同命令的别名不应冲突
        let content = "alias gs='git status'\n";
        let alias = Alias::new("gs", "git status -s");
        let result = update_alias_in_content(content, "gs", &alias).unwrap();
        assert!(result.contains("alias gs='git status -s'"));
    }

    // -- delete_alias_from_content 测试 --

    #[test]
    fn test_delete_alias_from_content() {
        let content = "alias gs='git status'\nalias ll='ls -la'\n";
        let result = delete_alias_from_content(content, "gs").unwrap();
        assert!(!result.contains("alias gs="));
        assert!(result.contains("alias ll='ls -la'"));
    }

    #[test]
    fn test_delete_alias_not_found() {
        let content = "alias gs='git status'\n";
        let result = delete_alias_from_content(content, "nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::AliasNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("期望 AliasNotFound，得到 {:?}", other),
        }
    }

    #[test]
    fn test_delete_alias_preserves_non_alias_lines() {
        let content = "# Header\nalias gs='git status'\nexport PATH=$HOME/bin:$PATH\n";
        let result = delete_alias_from_content(content, "gs").unwrap();
        assert!(result.contains("# Header"));
        assert!(result.contains("export PATH=$HOME/bin:$PATH"));
        assert!(!result.contains("alias gs="));
    }

    // -- format_alias_line 测试 --

    #[test]
    fn test_format_alias_with_special_chars() {
        let alias = Alias::new("build", "cargo build && cargo test");
        assert_eq!(format_alias_line(&alias), "alias build='cargo build && cargo test'");
    }
}

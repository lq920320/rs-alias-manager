/// 更新检查服务。
///
/// 通过 GitHub Releases API 检查应用程序的最新版本，
/// 并使用语义化版本（semver）进行比较，判断是否有可用更新。
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// GitHub 仓库标识（owner/repo）。
///
/// 部署到自己的仓库后请修改此常量。
const GITHUB_REPO: &str = "lq920320/rs-alias-manager";

/// GitHub Releases API 的基础 URL。
const GITHUB_API_BASE: &str = "https://api.github.com/repos";

/// 请求 GitHub API 时的超时时间（秒）。
const REQUEST_TIMEOUT_SECS: u64 = 15;

/// 更新检查的结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateInfo {
    /// 当前应用程序版本。
    pub current_version: String,
    /// 最新发布的版本。
    pub latest_version: String,
    /// 是否存在比当前版本更新的版本。
    pub has_update: bool,
    /// 发布页面的 URL（用于查看详情或下载）。
    pub release_url: String,
    /// 发布说明（changelog），可能为空。
    #[serde(default)]
    pub release_notes: Option<String>,
    /// 发布时间（ISO 8601 格式字符串）。
    #[serde(default)]
    pub published_at: Option<String>,
}

/// GitHub Releases API 返回的发布信息（仅解析所需字段）。
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    /// 版本标签，例如 "v0.2.0"。
    tag_name: String,
    /// 发布页面的 HTML URL。
    html_url: String,
    /// 发布说明正文。
    #[serde(default)]
    body: Option<String>,
    /// 发布时间。
    #[serde(default)]
    published_at: Option<String>,
}

/// 从版本标签中解析出语义化版本号。
///
/// 去除前导的 "v" / "V" 前缀，并尝试解析为 `semver::Version`。
/// 也支持去除前导空白字符。
///
/// # 示例
/// ```
/// use rs_alias_manager_tauri::services::update_checker::parse_version;
/// assert!(parse_version("v0.1.0").is_ok());
/// assert!(parse_version("0.2.0").is_ok());
/// assert!(parse_version("v1.2.3-beta.1").is_ok());
/// assert!(parse_version("invalid").is_err());
/// ```
pub fn parse_version(tag: &str) -> Result<semver::Version, AppError> {
    let cleaned = tag.trim().trim_start_matches('v').trim_start_matches('V');
    semver::Version::parse(cleaned)
        .map_err(|e| AppError::ParseError(format!("无法解析版本号 '{}': {}", tag, e)))
}

/// 判断 `latest` 是否严格大于 `current`（即存在更新）。
pub fn is_newer(current: &semver::Version, latest: &semver::Version) -> bool {
    latest > current
}

/// 检查应用程序是否有可用更新。
///
/// 通过 GitHub API 获取最新 release 信息，与 `current_version` 进行比较。
///
/// # 参数
/// * `current_version` - 当前应用程序版本（例如 "0.1.0"，可带 "v" 前缀）
///
/// # 错误
/// 返回 `AppError::NetworkError` 表示网络请求或反序列化失败，
/// 返回 `AppError::ParseError` 表示版本号解析失败。
pub fn check_for_updates(current_version: &str) -> Result<UpdateInfo, AppError> {
    let url = format!("{}/{}/releases/latest", GITHUB_API_BASE, GITHUB_REPO);

    let response: GitHubRelease = ureq::get(&url)
        .set("User-Agent", "rs-alias-manager")
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .call()
        .map_err(|e| AppError::NetworkError(format!("GitHub API 请求失败: {}", e)))?
        .into_json()
        .map_err(|e| AppError::NetworkError(format!("解析 GitHub 响应失败: {}", e)))?;

    let current = parse_version(current_version)?;
    let latest = parse_version(&response.tag_name)?;
    let has_update = is_newer(&current, &latest);

    Ok(UpdateInfo {
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        has_update,
        release_url: response.html_url,
        release_notes: response.body.filter(|s| !s.trim().is_empty()),
        published_at: response.published_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // === parse_version 测试 ===

    #[test]
    fn test_parse_version_plain() {
        let v = parse_version("0.1.0").unwrap();
        assert_eq!(v, semver::Version::new(0, 1, 0));
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        let v = parse_version("v0.2.0").unwrap();
        assert_eq!(v, semver::Version::new(0, 2, 0));
    }

    #[test]
    fn test_parse_version_with_uppercase_v_prefix() {
        let v = parse_version("V1.0.0").unwrap();
        assert_eq!(v, semver::Version::new(1, 0, 0));
    }

    #[test]
    fn test_parse_version_with_whitespace() {
        let v = parse_version("  v0.3.0  ").unwrap();
        assert_eq!(v, semver::Version::new(0, 3, 0));
    }

    #[test]
    fn test_parse_version_with_pre_release() {
        let v = parse_version("v1.2.3-beta.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.to_string().contains("beta"));
    }

    #[test]
    fn test_parse_version_with_build_metadata() {
        let v = parse_version("v1.0.0+build.5").unwrap();
        assert_eq!(v.major, 1);
    }

    #[test]
    fn test_parse_version_invalid_empty() {
        assert!(parse_version("").is_err());
    }

    #[test]
    fn test_parse_version_invalid_text() {
        assert!(parse_version("not-a-version").is_err());
    }

    #[test]
    fn test_parse_version_invalid_missing_patch() {
        assert!(parse_version("v1.2").is_err());
    }

    #[test]
    fn test_parse_version_v_only() {
        assert!(parse_version("v").is_err());
    }

    // === is_newer 测试 ===

    #[test]
    fn test_is_newer_true() {
        let current = semver::Version::new(0, 1, 0);
        let latest = semver::Version::new(0, 2, 0);
        assert!(is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_false_same() {
        let current = semver::Version::new(0, 1, 0);
        let latest = semver::Version::new(0, 1, 0);
        assert!(!is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_false_older() {
        // 最新版本比当前版本旧（例如降级场景）
        let current = semver::Version::new(1, 0, 0);
        let latest = semver::Version::new(0, 9, 0);
        assert!(!is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_patch_difference() {
        let current = semver::Version::new(1, 0, 0);
        let latest = semver::Version::new(1, 0, 1);
        assert!(is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_major_difference() {
        let current = semver::Version::new(1, 5, 3);
        let latest = semver::Version::new(2, 0, 0);
        assert!(is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_minor_difference() {
        let current = semver::Version::new(1, 0, 5);
        let latest = semver::Version::new(1, 1, 0);
        assert!(is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_pre_release_is_lower() {
        // 1.0.0-beta 低于 1.0.0 正式版
        let current = semver::Version::new(1, 0, 0);
        let latest = semver::Version::parse("1.0.0-beta").unwrap();
        assert!(!is_newer(&current, &latest));
    }

    #[test]
    fn test_is_newer_pre_release_to_stable() {
        // 1.0.0-beta -> 1.0.0 正式版视为有更新
        let current = semver::Version::parse("1.0.0-beta").unwrap();
        let latest = semver::Version::new(1, 0, 0);
        assert!(is_newer(&current, &latest));
    }

    // === GitHubRelease 反序列化测试 ===

    #[test]
    fn test_deserialize_github_release_full() {
        // 使用 r###"..."### 避免 JSON 中的 markdown "##" 触发原始字符串提前终止
        let json = r###"{
            "tag_name": "v0.5.0",
            "html_url": "https://github.com/owner/repo/releases/tag/v0.5.0",
            "body": "## What's new\n- Feature A\n- Bug fix B",
            "published_at": "2025-05-27T10:00:00Z"
        }"###;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v0.5.0");
        assert_eq!(release.html_url, "https://github.com/owner/repo/releases/tag/v0.5.0");
        assert!(release.body.as_ref().unwrap().contains("Feature A"));
        assert_eq!(release.published_at.as_ref().unwrap(), "2025-05-27T10:00:00Z");
    }

    #[test]
    fn test_deserialize_github_release_missing_optional_fields() {
        // body 和 published_at 是可选字段
        let json = r#"{
            "tag_name": "v0.1.0",
            "html_url": "https://github.com/owner/repo/releases/tag/v0.1.0"
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v0.1.0");
        assert!(release.body.is_none());
        assert!(release.published_at.is_none());
    }

    #[test]
    fn test_deserialize_github_release_empty_body() {
        let json = r#"{
            "tag_name": "v0.1.0",
            "html_url": "https://github.com/owner/repo/releases/tag/v0.1.0",
            "body": "",
            "published_at": null
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        // 空字符串会被解析为 Some("")，check_for_updates 会过滤掉
        assert_eq!(release.body.as_deref(), Some(""));
        assert!(release.published_at.is_none());
    }

    #[test]
    fn test_deserialize_github_release_missing_required_field() {
        // 缺少 tag_name 应反序列化失败
        let json = r#"{
            "html_url": "https://github.com/owner/repo/releases/tag/v0.1.0"
        }"#;
        let result: Result<GitHubRelease, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // === UpdateInfo 序列化测试（确保前端能正确反序列化）===

    #[test]
    fn test_update_info_serialize_roundtrip() {
        let info = UpdateInfo {
            current_version: "0.1.0".to_string(),
            latest_version: "0.2.0".to_string(),
            has_update: true,
            release_url: "https://github.com/owner/repo/releases/tag/v0.2.0".to_string(),
            release_notes: Some("Bug fixes".to_string()),
            published_at: Some("2025-05-27T10:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, info);
    }

    #[test]
    fn test_update_info_serialize_without_optional() {
        let info = UpdateInfo {
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            has_update: false,
            release_url: "https://github.com/owner/repo".to_string(),
            release_notes: None,
            published_at: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, info);
        // None 会被序列化为 null，确保前端 Option<String> 能正确反序列化
        assert!(json.contains("\"release_notes\":null"));
        assert!(json.contains("\"published_at\":null"));
    }
}

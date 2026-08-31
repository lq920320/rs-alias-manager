/// 前端的 Tauri invoke 命令包装器。
///
/// 提供类型安全的异步函数，通过 `window.__TAURI__.core.invoke()` 调用 Tauri 后端命令。
/// 在 Tauri 外部运行时（例如 `trunk serve`）回退到空/默认数据。
///
/// 错误统一为 [`CommandError`]：后端 `AppError` 序列化为 `{ code, message, detail }`，
/// 前端按 `code` 分支处理（如 `alias_conflict`），展示文案由 i18n 按码本地化。
use crate::state::app_state::{Alias, AppSettings};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// 后端命令返回的结构化错误。
///
/// `code` 为稳定的机器可读错误码（对应后端 `AppError::code()`），
/// `message` 为英文兜底文案，`detail` 携带上下文（如别名名称、文件路径）。
#[derive(Debug, Clone)]
pub struct CommandError {
    /// 机器可读错误码，如 `alias_conflict`、`config_not_found`。
    pub code: String,
    /// 后端返回的兜底文案（英文）。
    pub message: String,
    /// 错误上下文（如别名名称、文件路径）。
    pub detail: Option<String>,
}

impl CommandError {
    /// 构造内部错误（invoke 桥接本身的故障）。
    fn internal(message: impl Into<String>) -> Self {
        Self { code: "internal".to_string(), message: message.into(), detail: None }
    }

    /// 返回本地化后的展示文案。
    ///
    /// 查找 `error.{code}` 翻译键（`{}` 占位符用 `detail` 替换）；
    /// 找不到翻译时回退到后端 `message`。
    /// 必须在提供 `Locale` 上下文的响应式环境中调用。
    pub fn display(&self) -> String {
        let fallback = || match &self.detail {
            Some(d) => format!("{} ({})", self.message, d),
            None => self.message.clone(),
        };
        // 桥接层内部错误与未知错误没有本地化键，直接展示兜底文案
        if self.code == "internal" || self.code == "unknown" {
            return fallback();
        }
        let key = format!("error.{}", self.code);
        let translated = crate::i18n::t(&key);
        if translated == "[missing]" {
            return fallback();
        }
        match &self.detail {
            Some(d) => translated.replace("{}", d),
            // 无上下文数据时移除残留占位符
            None => translated
                .replace("{}", "")
                .trim()
                .trim_end_matches(':')
                .trim()
                .to_string(),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(d) => write!(f, "[{}] {} ({})", self.code, self.message, d),
            None => write!(f, "[{}] {}", self.code, self.message),
        }
    }
}

/// 从 Tauri invoke 的拒绝值中提取结构化错误。
///
/// 后端 `AppError` 序列化为 `{ code, message, detail }` 对象；
/// 若拒绝值是字符串或其他形式，则退化为 `unknown` 码并保留原文。
fn command_error_from_js(val: &JsValue) -> CommandError {
    if let Some(s) = val.as_string() {
        return CommandError { code: "unknown".to_string(), message: s, detail: None };
    }
    let get_field = |key: &str| -> Option<String> {
        js_sys::Reflect::get(val, &JsValue::from_str(key))
            .ok()
            .and_then(|v| v.as_string())
    };
    let code = get_field("code").unwrap_or_else(|| "unknown".to_string());
    let message = get_field("message").unwrap_or_else(|| format!("{:?}", val));
    let detail = get_field("detail");
    CommandError { code, message, detail }
}

/// 检查应用程序是否在 Tauri 环境中运行。
///
/// 注意 `Reflect::get` 对不存在的属性返回 `Ok(undefined)` 而非 `Err`，
/// 因此必须显式排除 `undefined`/`null`。
fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false)
}

/// 序列化 invoke 参数。
fn serialize_args<T: serde::Serialize>(args: &T) -> Result<JsValue, CommandError> {
    serde_wasm_bindgen::to_value(args)
        .map_err(|e| CommandError::internal(format!("serialize args failed: {}", e)))
}

/// 使用给定的名称和参数调用 Tauri 后端命令。
///
/// 使用全局的 `window.__TAURI__.core.invoke()` 函数。
async fn invoke<T: serde::de::DeserializeOwned>(cmd: &str, args: JsValue) -> Result<T, CommandError> {
    let window = web_sys::window()
        .ok_or_else(|| CommandError::internal("could not get window object"))?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| CommandError::internal("could not get __TAURI__ object"))?;
    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| CommandError::internal("could not get __TAURI__.core object"))?;
    let invoke_fn = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| CommandError::internal("could not get __TAURI__.core.invoke function"))?;

    let invoke_fn = js_sys::Function::from(invoke_fn);

    // 在 Tauri v2 中，invoke(cmd, args) 其中 args 是一个对象
    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(cmd), &args)
        .map_err(|e| command_error_from_js(&e))?;

    let js_val = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| command_error_from_js(&e))?;

    serde_wasm_bindgen::from_value(js_val)
        .map_err(|e| CommandError::internal(format!("deserialization failed: {}", e)))
}

/// 列出当前 Shell 配置文件中的所有别名。
pub async fn list_aliases() -> Result<Vec<Alias>, CommandError> {
    if !is_tauri() {
        return Ok(vec![]);
    }
    invoke::<Vec<Alias>>("list_aliases", JsValue::NULL).await
}

/// 添加新别名。
pub async fn add_alias(name: String, command: String, tags: Vec<String>) -> Result<(), CommandError> {
    if !is_tauri() {
        log::info!("[mock] add_alias: {} -> {}", name, command);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    struct Args {
        name: String,
        command: String,
        tags: Vec<String>,
    }
    let args = serialize_args(&Args { name, command, tags })?;
    invoke::<()>("add_alias", args).await
}

/// 更新现有别名。
pub async fn update_alias(
    old_name: String,
    name: String,
    command: String,
    tags: Vec<String>,
) -> Result<(), CommandError> {
    if !is_tauri() {
        log::info!("[mock] update_alias: {} -> {}", old_name, name);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        old_name: String,
        name: String,
        command: String,
        tags: Vec<String>,
    }
    let args = serialize_args(&Args { old_name, name, command, tags })?;
    invoke::<()>("update_alias", args).await
}

/// 按名称删除别名。
pub async fn delete_alias(name: String) -> Result<(), CommandError> {
    if !is_tauri() {
        log::info!("[mock] delete_alias: {}", name);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    struct Args {
        name: String,
    }
    let args = serialize_args(&Args { name })?;
    invoke::<()>("delete_alias", args).await
}

/// 检测当前 Shell 类型。
pub async fn detect_shell() -> Result<AppSettings, CommandError> {
    if !is_tauri() {
        return Ok(AppSettings::default());
    }
    invoke::<AppSettings>("detect_shell", JsValue::NULL).await
}

/// 列出可用模板，可选择按分类过滤。
pub async fn list_templates(
    category: Option<String>,
) -> Result<Vec<crate::state::app_state::Template>, CommandError> {
    if !is_tauri() {
        return Ok(vec![]);
    }
    #[derive(serde::Serialize)]
    struct Args {
        category: Option<String>,
    }
    let args = serialize_args(&Args { category })?;
    invoke::<Vec<crate::state::app_state::Template>>("list_templates", args).await
}

/// 模板导入结果。
#[derive(Deserialize)]
pub struct ImportResult {
    /// 成功导入的模板数量。
    pub imported: usize,
    /// 因别名已存在而跳过的数量。
    pub skipped: usize,
}

/// 按名称导入选中的模板。
pub async fn import_templates(names: Vec<String>) -> Result<ImportResult, CommandError> {
    if !is_tauri() {
        log::info!("[mock] import_templates: {:?}", names);
        return Ok(ImportResult { imported: names.len(), skipped: 0 });
    }
    #[derive(serde::Serialize)]
    struct Args {
        names: Vec<String>,
    }
    let args = serialize_args(&Args { names })?;
    invoke::<ImportResult>("import_templates", args).await
}

/// 获取当前应用程序设置。
pub async fn get_settings() -> Result<AppSettings, CommandError> {
    if !is_tauri() {
        return Ok(AppSettings::default());
    }
    invoke::<AppSettings>("get_settings", JsValue::NULL).await
}

/// 更新应用程序设置。
pub async fn update_settings(
    shell_type: Option<String>,
    custom_config_path: Option<String>,
    auto_refresh: Option<bool>,
    instant_apply: Option<bool>,
    locale: Option<String>,
) -> Result<AppSettings, CommandError> {
    if !is_tauri() {
        log::info!("[mock] update_settings");
        return Ok(AppSettings::default());
    }
    #[derive(serde::Serialize)]
    struct Args {
        shell_type: Option<String>,
        custom_config_path: Option<String>,
        auto_refresh: Option<bool>,
        instant_apply: Option<bool>,
        locale: Option<String>,
    }
    let args = serialize_args(&Args { shell_type, custom_config_path, auto_refresh, instant_apply, locale })?;
    invoke::<AppSettings>("update_settings", args).await
}

/// 获取有效的配置文件路径。
pub async fn get_config_file_path() -> Result<String, CommandError> {
    if !is_tauri() {
        return Ok("~/.zshrc".to_string());
    }
    invoke::<String>("get_config_file_path", JsValue::NULL).await
}

/// 批量操作的结果。
#[derive(Deserialize)]
pub struct BatchResult {
    /// 成功操作的数量。
    pub success_count: usize,
    /// 因别名已存在而跳过的数量。
    pub skipped_count: usize,
    /// 失败的错误信息列表。
    pub errors: Vec<String>,
}

/// 批量添加别名。
pub async fn batch_add_aliases(aliases: Vec<Alias>) -> Result<BatchResult, CommandError> {
    if !is_tauri() {
        log::info!("[mock] batch_add_aliases: {} items", aliases.len());
        return Ok(BatchResult { success_count: aliases.len(), skipped_count: 0, errors: vec![] });
    }
    #[derive(serde::Serialize)]
    struct Args {
        aliases: Vec<Alias>,
    }
    let args = serialize_args(&Args { aliases })?;
    invoke::<BatchResult>("batch_add_aliases", args).await
}

/// 批量删除别名。
pub async fn batch_delete_aliases(names: Vec<String>) -> Result<BatchResult, CommandError> {
    if !is_tauri() {
        log::info!("[mock] batch_delete_aliases: {:?}", names);
        return Ok(BatchResult { success_count: names.len(), skipped_count: 0, errors: vec![] });
    }
    #[derive(serde::Serialize)]
    struct Args {
        names: Vec<String>,
    }
    let args = serialize_args(&Args { names })?;
    invoke::<BatchResult>("batch_delete_aliases", args).await
}

/// 更新检查结果。
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UpdateInfo {
    /// 当前应用程序版本。
    pub current_version: String,
    /// 最新发布的版本。
    pub latest_version: String,
    /// 是否存在比当前版本更新的版本。
    pub has_update: bool,
    /// 发布页面的 URL。
    pub release_url: String,
    /// 发布说明（changelog），可能为空。
    pub release_notes: Option<String>,
    /// 发布时间（ISO 8601 格式字符串）。
    pub published_at: Option<String>,
}

/// 检查应用程序是否有可用更新。
///
/// 通过后端调用 GitHub Releases API 获取最新版本并与当前版本比较。
/// 在 Tauri 环境外运行时返回一个「已是最新」的默认结果。
pub async fn check_for_updates() -> Result<UpdateInfo, CommandError> {
    if !is_tauri() {
        log::info!("[mock] check_for_updates");
        return Ok(UpdateInfo {
            current_version: "0.0.0".to_string(),
            latest_version: "0.0.0".to_string(),
            has_update: false,
            release_url: String::new(),
            release_notes: None,
            published_at: None,
        });
    }
    invoke::<UpdateInfo>("check_for_updates", JsValue::NULL).await
}

/// 获取当前应用程序版本号。
pub async fn get_app_version() -> Result<String, CommandError> {
    if !is_tauri() {
        return Ok("0.0.0".to_string());
    }
    invoke::<String>("get_app_version", JsValue::NULL).await
}

/// 对当前配置文件执行 source，使别名变更尽快对新终端生效。
///
/// 仅在 Tauri 环境内调用。前端应在增删改别名成功且 `instant_apply` 开启时调用。
pub async fn auto_source(shell_type: String) -> Result<(), CommandError> {
    if !is_tauri() {
        log::info!("[mock] auto_source: {}", shell_type);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    struct Args {
        shell_type: String,
    }
    let args = serialize_args(&Args { shell_type })?;
    invoke::<()>("auto_source", args).await
}

/// 保存（新增或覆盖）一个用户自定义模板。
pub async fn save_template(template: crate::state::app_state::Template) -> Result<(), CommandError> {
    if !is_tauri() {
        log::info!("[mock] save_template: {}", template.name);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    struct Args {
        template: crate::state::app_state::Template,
    }
    let args = serialize_args(&Args { template })?;
    invoke::<()>("save_template", args).await
}

/// 删除一个用户自定义模板。
pub async fn delete_template(name: String) -> Result<(), CommandError> {
    if !is_tauri() {
        log::info!("[mock] delete_template: {}", name);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    struct Args {
        name: String,
    }
    let args = serialize_args(&Args { name })?;
    invoke::<()>("delete_template", args).await
}

/// 单条配置文件备份元数据（与后端 `BackupEntry` 对应）。
#[derive(Debug, Clone, Deserialize)]
pub struct BackupEntry {
    /// 唯一标识（毫秒时间戳 + 进程号）。
    pub id: String,
    /// 备份时配置文件的路径。
    pub original_path: String,
    /// 创建时间（Unix 毫秒）。
    pub created_at: u64,
    /// 备份内容字节数。
    pub size: u64,
}

/// 列出所有配置文件备份（最新在前）。
pub async fn list_backups() -> Result<Vec<BackupEntry>, CommandError> {
    if !is_tauri() {
        return Ok(vec![]);
    }
    invoke::<Vec<BackupEntry>>("list_backups", JsValue::NULL).await
}

/// 将指定备份恢复到其原始配置文件路径。
///
/// 后端在恢复前会先备份当前配置，确保回滚本身可撤销。
pub async fn restore_backup(id: String) -> Result<BackupEntry, CommandError> {
    if !is_tauri() {
        log::info!("[mock] restore_backup: {}", id);
        return Ok(BackupEntry { id, original_path: String::new(), created_at: 0, size: 0 });
    }
    #[derive(serde::Serialize)]
    struct Args {
        id: String,
    }
    let args = serialize_args(&Args { id })?;
    invoke::<BackupEntry>("restore_backup", args).await
}

/// 订阅后端配置变更事件（"config-changed"）。
///
/// 后端在配置文件被外部编辑且 `auto_refresh` 开启时广播该事件。
/// 回调在事件到达时被调用（前端需自行防抖）。
pub fn listen_config_changed(callback: impl Fn(()) + 'static) {
    if !is_tauri() {
        return;
    }
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let tauri = match js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) {
        Ok(t) => t,
        Err(_) => return,
    };
    let event = match js_sys::Reflect::get(&tauri, &JsValue::from_str("event")) {
        Ok(e) => e,
        Err(_) => return,
    };
    let listen_fn = match js_sys::Reflect::get(&event, &JsValue::from_str("listen")) {
        Ok(f) => f,
        Err(_) => return,
    };
    let listen_fn = match listen_fn.dyn_ref::<js_sys::Function>() {
        Some(f) => f,
        None => return,
    };
    let closure = Closure::wrap(Box::new(move |_payload: JsValue| {
        callback(());
    }) as Box<dyn Fn(JsValue)>);
    let _ = listen_fn.call2(
        &event,
        &JsValue::from_str("config-changed"),
        &closure.into_js_value(),
    );
    // Closure 被 Tauri 持有直到页面卸载；此处不主动释放。
}

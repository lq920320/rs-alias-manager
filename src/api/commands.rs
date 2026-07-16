/// 前端的 Tauri invoke 命令包装器。
///
/// 提供类型安全的异步函数，通过 `window.__TAURI__.core.invoke()` 调用 Tauri 后端命令。
/// 在 Tauri 外部运行时（例如 `trunk serve`）回退到空/默认数据。
use crate::state::app_state::{Alias, AppSettings};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// 检查应用程序是否在 Tauri 环境中运行。
fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
        .is_some()
}

/// 使用给定的名称和参数调用 Tauri 后端命令。
///
/// 使用全局的 `window.__TAURI__.core.invoke()` 函数。
async fn invoke<T: serde::de::DeserializeOwned>(cmd: &str, args: JsValue) -> Result<T, String> {
    let window = web_sys::window().ok_or("could not get window object")?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "could not get __TAURI__ object")?;
    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "could not get __TAURI__.core object")?;
    let invoke_fn = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "could not get __TAURI__.core.invoke function")?;

    let invoke_fn = js_sys::Function::from(invoke_fn);

    // 在 Tauri v2 中，invoke(cmd, args) 其中 args 是一个对象
    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(cmd), &args)
        .map_err(|e| format!("invoke call failed: {:?}", e))?;

    let js_val = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| format!("invoke async failed: {:?}", e))?;

    serde_wasm_bindgen::from_value(js_val).map_err(|e| format!("deserialization failed: {}", e))
}

/// 列出当前 Shell 配置文件中的所有别名。
pub async fn list_aliases() -> Result<Vec<Alias>, String> {
    if !is_tauri() {
        return Ok(vec![]);
    }
    invoke::<Vec<Alias>>("list_aliases", JsValue::NULL).await
}

/// 添加新别名。
pub async fn add_alias(name: String, command: String, tags: Vec<String>) -> Result<(), String> {
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
    let args = serde_wasm_bindgen::to_value(&Args { name, command, tags })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<()>("add_alias", args).await
}

/// 更新现有别名。
pub async fn update_alias(
    old_name: String,
    name: String,
    command: String,
    tags: Vec<String>,
) -> Result<(), String> {
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
    let args = serde_wasm_bindgen::to_value(&Args { old_name, name, command, tags })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<()>("update_alias", args).await
}

/// 按名称删除别名。
pub async fn delete_alias(name: String) -> Result<(), String> {
    if !is_tauri() {
        log::info!("[mock] delete_alias: {}", name);
        return Ok(());
    }
    #[derive(serde::Serialize)]
    struct Args {
        name: String,
    }
    let args = serde_wasm_bindgen::to_value(&Args { name })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<()>("delete_alias", args).await
}

/// 检测当前 Shell 类型。
pub async fn detect_shell() -> Result<AppSettings, String> {
    if !is_tauri() {
        return Ok(AppSettings::default());
    }
    invoke::<AppSettings>("detect_shell", JsValue::NULL).await
}

/// 列出可用模板，可选择按分类过滤。
pub async fn list_templates(
    category: Option<String>,
) -> Result<Vec<crate::state::app_state::Template>, String> {
    if !is_tauri() {
        return Ok(vec![]);
    }
    #[derive(serde::Serialize)]
    struct Args {
        category: Option<String>,
    }
    let args = serde_wasm_bindgen::to_value(&Args { category })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<Vec<crate::state::app_state::Template>>("list_templates", args).await
}

/// 按名称导入选中的模板。
pub async fn import_templates(names: Vec<String>) -> Result<usize, String> {
    if !is_tauri() {
        log::info!("[mock] import_templates: {:?}", names);
        return Ok(names.len());
    }
    #[derive(serde::Serialize)]
    struct Args {
        names: Vec<String>,
    }
    let args = serde_wasm_bindgen::to_value(&Args { names })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<usize>("import_templates", args).await
}

/// 获取当前应用程序设置。
pub async fn get_settings() -> Result<AppSettings, String> {
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
    locale: Option<String>,
) -> Result<AppSettings, String> {
    if !is_tauri() {
        log::info!("[mock] update_settings");
        return Ok(AppSettings::default());
    }
    #[derive(serde::Serialize)]
    struct Args {
        shell_type: Option<String>,
        custom_config_path: Option<String>,
        auto_refresh: Option<bool>,
        locale: Option<String>,
    }
    let args = serde_wasm_bindgen::to_value(&Args { shell_type, custom_config_path, auto_refresh, locale })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<AppSettings>("update_settings", args).await
}

/// 获取有效的配置文件路径。
pub async fn get_config_file_path() -> Result<String, String> {
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
    /// 失败的错误信息列表。
    pub errors: Vec<String>,
}

/// 批量添加别名。
pub async fn batch_add_aliases(aliases: Vec<Alias>) -> Result<BatchResult, String> {
    if !is_tauri() {
        log::info!("[mock] batch_add_aliases: {} items", aliases.len());
        return Ok(BatchResult { success_count: aliases.len(), errors: vec![] });
    }
    #[derive(serde::Serialize)]
    struct Args {
        aliases: Vec<Alias>,
    }
    let args = serde_wasm_bindgen::to_value(&Args { aliases })
        .map_err(|e| format!("serialize args failed: {}", e))?;
    invoke::<BatchResult>("batch_add_aliases", args).await
}

/// 批量删除别名。
pub async fn batch_delete_aliases(names: Vec<String>) -> Result<BatchResult, String> {
    if !is_tauri() {
        log::info!("[mock] batch_delete_aliases: {:?}", names);
        return Ok(BatchResult { success_count: names.len(), errors: vec![] });
    }
    #[derive(serde::Serialize)]
    struct Args {
        names: Vec<String>,
    }
    let args = serde_wasm_bindgen::to_value(&Args { names })
        .map_err(|e| format!("serialize args failed: {}", e))?;
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
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
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
pub async fn get_app_version() -> Result<String, String> {
    if !is_tauri() {
        return Ok("0.0.0".to_string());
    }
    invoke::<String>("get_app_version", JsValue::NULL).await
}

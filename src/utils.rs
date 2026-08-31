/// 前端共用工具函数。

/// 在指定延迟后执行闭包（基于浏览器 setTimeout）。
pub fn set_timeout(f: impl FnOnce() + 'static, dur: std::time::Duration) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    if let Some(window) = web_sys::window() {
        let cb = Closure::once_into_js(move || f());
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.unchecked_ref(),
            dur.as_millis() as i32,
        );
    }
}

/// 安全地触发文件下载（创建临时 anchor 元素）。
///
/// 如果浏览器环境不可用则静默失败而非 panic。
pub fn trigger_download(filename: &str, content: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };
    let body = match document.body() {
        Some(b) => b,
        None => return,
    };

    let anchor = match document.create_element("a") {
        Ok(el) => el,
        Err(_) => return,
    };

    use wasm_bindgen::JsCast;
    let anchor: web_sys::HtmlElement = match anchor.dyn_into() {
        Ok(el) => el,
        Err(_) => return,
    };

    let href = format!(
        "data:application/json;charset=utf-8,{}",
        js_sys::encode_uri_component(content)
    );

    let _ = anchor.set_attribute("href", &href);
    let _ = anchor.set_attribute("download", filename);
    let _ = anchor.set_attribute("style", "display:none");
    let _ = body.append_child(&anchor);
    anchor.click();
    let _ = body.remove_child(&anchor);
}

/// 更新 HTML 文档的 lang 属性。
pub fn set_html_lang(lang: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(html) = doc.query_selector("html").ok().flatten() {
                let _ = html.set_attribute("lang", lang);
            }
        }
    }
}

/// 按标签名哈希返回稳定的颜色类，保证同名标签在界面各处颜色一致。
pub fn tag_color_class(tag: &str) -> &'static str {
    const COLORS: [&str; 6] = [
        "tag--blue",
        "tag--green",
        "tag--purple",
        "tag--orange",
        "tag--pink",
        "tag--cyan",
    ];
    let mut hash: u32 = 5381;
    for b in tag.as_bytes() {
        hash = hash.wrapping_mul(33) ^ (u32::from(*b));
    }
    COLORS[(hash % COLORS.len() as u32) as usize]
}

/// 校验别名名称，返回本地化错误信息。
///
/// 规则与后端 `Alias::validate_name` 保持一致：非空、不以连字符开头、
/// 仅含字母数字下划线连字符。须在提供 `Locale` 上下文的环境中调用。
pub fn validate_alias_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err(crate::i18n::t("validate.name_empty"));
    }
    if name.starts_with('-') {
        return Err(crate::i18n::t("validate.name_hyphen"));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(crate::i18n::t("validate.name_chars"));
    }
    Ok(())
}

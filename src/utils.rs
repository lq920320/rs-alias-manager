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

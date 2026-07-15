/// 侧边栏导航组件。
///
/// 提供三个主要板块的导航链接：
/// - 别名管理
/// - 模板库
/// - 设置
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::i18n::t;
use crate::state::app_state::AppState;

fn set_theme_on_html(dark: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(html) = doc.query_selector("html").ok().flatten() {
                if dark {
                    html.set_attribute("data-theme", "dark").ok();
                } else {
                    html.remove_attribute("data-theme").ok();
                }
            }
        }
    }
}

fn save_theme_preference(dark: bool) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            storage.set("theme", if dark { "dark" } else { "light" }).ok();
        }
    }
}

fn load_saved_theme() -> Option<bool> {
    let window = web_sys::window()?;
    // 首先检查 localStorage
    let stored = window.local_storage().ok().flatten().and_then(|s| s.get("theme").ok()).flatten();
    match stored.as_deref() {
        Some("dark") => return Some(true),
        Some("light") => return Some(false),
        _ => {},
    }
    // 回退到系统偏好
    window.match_media("(prefers-color-scheme: dark)").ok().flatten().map(|m| m.matches())
}

/// 侧边栏导航组件。
#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");
    let location = use_location();

    // 主题状态："light" 或 "dark"
    let (is_dark, set_is_dark) = signal(load_saved_theme().unwrap_or(false));

    // 挂载时 / is_dark 变化时应用主题
    Effect::new(move || {
        set_theme_on_html(is_dark.get());
    });

    // 将主题变更应用到 DOM 和 localStorage
    let toggle_theme = move |_| {
        let new_dark = !is_dark.get();
        set_is_dark.set(new_dark);
        set_theme_on_html(new_dark);
        save_theme_preference(new_dark);
    };

    // 判断路径是否为当前活动的路由
    let is_active = |path: &str, current: &str| -> bool {
        match path {
            "/" => current == "/" || current == "/aliases",
            _ => current.starts_with(path),
        }
    };

    view! {
        <nav class="sidebar">
            <div class="sidebar__logo">
                {move || t("app.title")}
            </div>
            <div class="sidebar__nav">
                {move || {
                    let loc = location.pathname.get();
                    let _locale = state.locale.get(); // subscribe to locale changes
                    let active = is_active("/", &loc);
                    view! {
                        <A href="/" attr:class=move || if active { "sidebar__nav-item sidebar__nav-item--active" } else { "sidebar__nav-item" }>
                            <span class="sidebar__nav-icon">"\u{26a1}"</span>
                            {t("nav.aliases")}
                        </A>
                    }
                }}
                {move || {
                    let loc = location.pathname.get();
                    let _locale = state.locale.get();
                    let active = is_active("/templates", &loc);
                    view! {
                        <A href="/templates" attr:class=move || if active { "sidebar__nav-item sidebar__nav-item--active" } else { "sidebar__nav-item" }>
                            <span class="sidebar__nav-icon">"\u{1f4e6}"</span>
                            {t("nav.templates")}
                        </A>
                    }
                }}
                {move || {
                    let loc = location.pathname.get();
                    let _locale = state.locale.get();
                    let active = is_active("/settings", &loc);
                    view! {
                        <A href="/settings" attr:class=move || if active { "sidebar__nav-item sidebar__nav-item--active" } else { "sidebar__nav-item" }>
                            <span class="sidebar__nav-icon">"\u{2699}"</span>
                            {t("nav.settings")}
                        </A>
                    }
                }}
            </div>
            <div class="sidebar__footer">
                <button class="theme-toggle" on:click=toggle_theme>
                    <span class="theme-toggle__icon">{move || if is_dark.get() { "\u{2600}" } else { "\u{1f319}" }}</span>
                    {move || if is_dark.get() { t("theme.light") } else { t("theme.dark") }}
                </button>
                {
                    move || {
                        let shell = state.shell_type.get();
                        let path = state.config_path.get();
                        view! {
                            <div style="margin-top:8px">
                                <div style="font-weight:600;opacity:0.7">{ format!("Shell: {}", shell.label()) }</div>
                                <div style="font-size:11px;word-break:break-all;margin-top:4px;opacity:0.5;line-height:1.4">
                                    { path }
                                </div>
                            </div>
                        }
                    }
                }
            </div>
        </nav>
    }
}

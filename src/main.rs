use leptos::prelude::*;

mod api;
mod app;
mod components;
pub mod i18n;
mod pages;
mod state;
pub mod utils;

use state::app_state::AppstateProvider;

/// Leptos 前端应用程序的入口点。
/// 将根 `App` 组件挂载到 `#app` DOM 元素，
/// 包裹在 `AppstateProvider` 中以提供全局状态。
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
    mount_to_body(|| {
        view! {
            <AppstateProvider>
                <app::App />
            </AppstateProvider>
        }
    })
}

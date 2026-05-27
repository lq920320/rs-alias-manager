use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::{Route, Router, Routes};

use crate::components::sidebar::Sidebar;
use crate::pages::{
    alias_page::AliasPage, settings_page::SettingsPage, template_page::TemplatePage,
};

/// 根应用程序组件。
/// 提供带有侧边栏和路由内容区域的整体布局。
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Title text="别名管理器" />
            <div class="app-layout">
                <Sidebar />
                <div class="app-main">
                    <Routes fallback=|| "Page not found">
                        <Route path=leptos_router::path!("/") view=AliasPage />
                        <Route path=leptos_router::path!("/aliases") view=AliasPage />
                        <Route path=leptos_router::path!("/templates") view=TemplatePage />
                        <Route path=leptos_router::path!("/settings") view=SettingsPage />
                    </Routes>
                </div>
            </div>
        </Router>
    }
}

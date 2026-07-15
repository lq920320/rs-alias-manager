use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::{Route, Router, Routes};

use crate::components::sidebar::Sidebar;
use crate::i18n::t;
use crate::pages::{
    alias_page::AliasPage, settings_page::SettingsPage, template_page::TemplatePage,
};

/// Root application component.
/// Provides the overall layout with a sidebar and routed content area.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Title text=move || t("app.title") />
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

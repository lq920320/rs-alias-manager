/// 搜索栏组件。
///
/// 提供用于过滤别名列表的实时搜索输入。
use leptos::prelude::*;

use crate::state::app_state::AppState;

/// 搜索栏组件。
#[component]
pub fn SearchBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");

    view! {
        <div class="search-bar">
            <span class="search-bar__icon">"🔍"</span>
            <input
                class="search-bar__input"
                type="text"
                placeholder="搜索别名..."
                prop:value=move || state.search_query.get()
                on:input=move |e| {
                    state.set_search_query.set(event_target_value(&e));
                }
            />
        </div>
    }
}

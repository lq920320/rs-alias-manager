/// Search bar component.
use leptos::prelude::*;

use crate::i18n::t;
use crate::state::app_state::AppState;

/// Search bar component.
#[component]
pub fn SearchBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");

    view! {
        <div class="search-bar">
            <span class="search-bar__icon">"🔍"</span>
            <input
                class="search-bar__input"
                type="text"
                placeholder=move || t("search.placeholder")
                prop:value=move || state.search_query.get()
                on:input=move |e| {
                    state.set_search_query.set(event_target_value(&e));
                }
            />
        </div>
    }
}

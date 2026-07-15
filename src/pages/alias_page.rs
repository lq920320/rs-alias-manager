/// 别名管理页面。
///
/// 提供查看、添加、编辑和删除 Shell 别名的主界面。
/// 包含搜索过滤和多选操作功能。
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::alias_form::AliasForm;
use crate::components::alias_list::AliasList;
use crate::components::search_bar::SearchBar;
use crate::i18n::t;
use crate::state::app_state::AppState;
use crate::utils::trigger_download;

/// 别名管理页面组件。
#[component]
pub fn AliasPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");
    let (show_form, set_show_form) = signal(false);
    let (editing_alias, set_editing_alias) = signal(None::<(String, String, Vec<String>)>);

    // 挂载时加载别名
    let _ = Effect::new(move || {
        state.load_aliases();
    });

    // 挂载时加载设置
    let _ = Effect::new(move || {
        state.load_settings_and_config();
    });

    let on_add_click = move || {
        set_editing_alias.set(None);
        set_show_form.set(true);
    };

    let on_edit = move |(name, command, tags): (String, String, Vec<String>)| {
        set_editing_alias.set(Some((name, command, tags)));
        set_show_form.set(true);
    };

    let on_delete = {
        let state = state;
        move |name: String| {
            let state = state;
            spawn_local(async move {
                state.set_loading.set(true);
                match crate::api::commands::delete_alias(name).await {
                    Ok(()) => {
                        // 重新加载别名
                        match crate::api::commands::list_aliases().await {
                            Ok(aliases) => {
                                state.set_aliases.set(aliases);
                            },
                            Err(e) => {
                                state.set_error_message.set(Some(e));
                            },
                        }
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e));
                    },
                }
                state.set_loading.set(false);
            });
        }
    };

    let on_form_submit = {
        let state = state;
        move |(old_name, name, command, tags): (Option<String>, String, String, Vec<String>)| {
            let state = state;
            spawn_local(async move {
                state.set_loading.set(true);
                state.set_error_message.set(None);
                let result = match old_name {
                    Some(old) => crate::api::commands::update_alias(old, name, command, tags).await,
                    None => crate::api::commands::add_alias(name, command, tags).await,
                };
                match result {
                    Ok(()) => {
                        set_show_form.set(false);
                        set_editing_alias.set(None);
                        // 重新加载别名
                        match crate::api::commands::list_aliases().await {
                            Ok(aliases) => {
                                state.set_aliases.set(aliases);
                            },
                            Err(e) => {
                                state.set_error_message.set(Some(e));
                            },
                        }
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e));
                    },
                }
                state.set_loading.set(false);
            });
        }
    };

    let on_form_cancel = move |_: ()| {
        set_show_form.set(false);
        set_editing_alias.set(None);
    };

    let on_export = {
        let state = state;
        move || {
            let aliases = state.aliases.get();
            let json = serde_json::to_string_pretty(&*aliases).unwrap_or_else(|_| "[]".to_string());
            trigger_download("aliases.json", &json);
        }
    };

    let on_import = {
        let state = state;
        move || {
            let state = state;
            spawn_local(async move {
                // 导入时使用基于 prompt 的简单方式
                let window = match web_sys::window() {
                    Some(w) => w,
                    None => return,
                };
                if let Ok(Some(json_str)) =
                    window.prompt_with_message_and_default(&t("alias.import_prompt"), "[]")
                {
                    match serde_json::from_str::<Vec<crate::state::app_state::Alias>>(&json_str) {
                        Ok(aliases) => {
                            state.set_loading.set(true);
                            match crate::api::commands::batch_add_aliases(aliases).await {
                                Ok(result) => {
                                    if !result.errors.is_empty() {
                                        state.set_error_message.set(Some(
                                            t("alias.import_partial")
                                                .replace("{}", &result.success_count.to_string())
                                                .replace("{}", &result.errors.join(", "))
                                        ));
                                    }
                                },
                                Err(e) => {
                                    state.set_error_message.set(Some(e));
                                },
                            }
                            // 重新加载
                            match crate::api::commands::list_aliases().await {
                                Ok(aliases) => state.set_aliases.set(aliases),
                                Err(e) => state.set_error_message.set(Some(e)),
                            }
                            state.set_loading.set(false);
                        },
                        Err(e) => {
                            state.set_error_message.set(Some(
                                t("alias.json_parse_error").replace("{}", &e.to_string())
                            ));
                        },
                    }
                }
            });
        }
    };

    let on_delete_selected = {
        let state = state;
        move |_: ()| {
            let state = state;
            let selected = state.selected_aliases.get();
            if selected.is_empty() {
                return;
            }
            spawn_local(async move {
                state.set_loading.set(true);
                match crate::api::commands::batch_delete_aliases(selected.to_vec()).await {
                    Ok(result) => {
                        if !result.errors.is_empty() {
                            state.set_error_message.set(Some(
                                t("alias.delete_partial").replace("{}", &result.errors.join(", "))
                            ));
                        }
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e));
                    },
                }
                state.set_selected_aliases.set(Vec::new());
                match crate::api::commands::list_aliases().await {
                    Ok(aliases) => state.set_aliases.set(aliases),
                    Err(e) => state.set_error_message.set(Some(e)),
                }
                state.set_loading.set(false);
            });
        }
    };

    view! {
        <div class="app-header">
            <div class="search-bar">
                <SearchBar />
            </div>
            <div class="app-header__actions">
                <button class="btn btn--secondary btn--sm" on:click=move |_| on_import()>
                    {move || t("alias.import")}
                </button>
                <button class="btn btn--secondary btn--sm" on:click=move |_| on_export()>
                    {move || t("alias.export")}
                </button>
                <button class="btn btn--primary" on:click=move |_| on_add_click()>
                    {move || t("alias.add_btn")}
                </button>
            </div>
        </div>

        <div class="app-content">
            {
                move || {
                    let err = state.error_message.get();
                    if let Some(e) = err {
                        view! {
                            <div class="alert alert--error mb-lg">
                                {e}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }
            }

            {
                move || {
                    let loading = state.loading.get();
                    if loading {
                        view! {
                            <div class="loading">
                                <div class="loading__spinner"></div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }
            }

            <AliasList
                on_edit=Callback::new(on_edit)
                on_delete=Callback::new(on_delete)
                on_delete_selected=Callback::new(on_delete_selected)
            />
        </div>

        {
            move || {
                let show = show_form.get();
                if show {
                    let editing = editing_alias.get();
                    view! {
                        <AliasForm
                            alias=editing
                            on_submit=Callback::new(on_form_submit)
                            on_cancel=Callback::new(on_form_cancel)
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }
        }
    }
}

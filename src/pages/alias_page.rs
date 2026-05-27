/// 别名管理页面。
///
/// 提供查看、添加、编辑和删除 Shell 别名的主界面。
/// 包含搜索过滤和多选操作功能。
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::components::alias_form::AliasForm;
use crate::components::alias_list::AliasList;
use crate::components::search_bar::SearchBar;
use crate::state::app_state::AppState;

/// 别名管理页面组件。
#[component]
pub fn AliasPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");
    let (show_form, set_show_form) = signal(false);
    let (editing_alias, set_editing_alias) = signal(None::<(String, String, Vec<String>)>);

    // 挂载时加载别名
    let load_aliases = {
        let state = state;
        move || {
            let state = state;
            spawn_local(async move {
                state.set_loading.set(true);
                state.set_error_message.set(None);
                match crate::api::commands::list_aliases().await {
                    Ok(aliases) => {
                        state.set_aliases.set(aliases);
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e));
                    },
                }
                state.set_loading.set(false);
            });
        }
    };

    // 挂载时加载
    let _ = Effect::new(move || {
        load_aliases();
    });

    // 挂载时加载设置
    let load_settings = {
        let state = state;
        move || {
            let state = state;
            spawn_local(async move {
                match crate::api::commands::get_settings().await {
                    Ok(settings) => {
                        state.set_shell_type.set(settings.shell_type);
                        state.set_settings.set(settings);
                    },
                    Err(e) => {
                        log::warn!("Failed to load settings: {}", e);
                    },
                }
                match crate::api::commands::get_config_file_path().await {
                    Ok(path) => {
                        state.set_config_path.set(path);
                    },
                    Err(e) => {
                        log::warn!("Failed to get config path: {}", e);
                    },
                }
            });
        }
    };

    let _ = Effect::new(move || {
        load_settings();
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
            // 使用 data URI 创建下载
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let anchor = document.create_element("a").unwrap();
            let anchor: web_sys::HtmlElement = anchor.dyn_into().unwrap();
            anchor
                .set_attribute(
                    "href",
                    &format!(
                        "data:application/json;charset=utf-8,{}",
                        js_sys::encode_uri_component(&json)
                    ),
                )
                .unwrap();
            anchor.set_attribute("download", "aliases.json").unwrap();
            anchor.set_attribute("style", "display:none").unwrap();
            let body = document.body().unwrap();
            body.append_child(&anchor).unwrap();
            anchor.click();
            let _ = body.remove_child(&anchor);
        }
    };

    let on_import = {
        let state = state;
        move || {
            let state = state;
            spawn_local(async move {
                // 导入时使用基于 prompt 的简单方式
                let window = web_sys::window().unwrap();
                if let Ok(Some(json_str)) =
                    window.prompt_with_message_and_default("请粘贴别名 JSON 数据:", "[]")
                {
                    match serde_json::from_str::<Vec<crate::state::app_state::Alias>>(&json_str) {
                        Ok(aliases) => {
                            state.set_loading.set(true);
                            let mut imported = 0usize;
                            let mut errors = Vec::new();
                            for alias in aliases {
                                match crate::api::commands::add_alias(
                                    alias.name.clone(),
                                    alias.command.clone(),
                                    alias.tags.clone(),
                                )
                                .await
                                {
                                    Ok(()) => imported += 1,
                                    Err(e) => errors.push(format!("{}: {}", alias.name, e)),
                                }
                            }
                            // 重新加载
                            match crate::api::commands::list_aliases().await {
                                Ok(aliases) => state.set_aliases.set(aliases),
                                Err(e) => state.set_error_message.set(Some(e)),
                            }
                            if !errors.is_empty() {
                                state.set_error_message.set(Some(format!(
                                    "导入了 {} 个别名，失败: {}",
                                    imported,
                                    errors.join(", ")
                                )));
                            }
                            state.set_loading.set(false);
                        },
                        Err(e) => {
                            state.set_error_message.set(Some(format!("JSON 解析失败: {}", e)));
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
                let mut errors = Vec::new();
                for name in selected.iter() {
                    if let Err(e) = crate::api::commands::delete_alias(name.clone()).await {
                        errors.push(format!("{}: {}", name, e));
                    }
                }
                state.set_selected_aliases.set(Vec::new());
                match crate::api::commands::list_aliases().await {
                    Ok(aliases) => state.set_aliases.set(aliases),
                    Err(e) => state.set_error_message.set(Some(e)),
                }
                if !errors.is_empty() {
                    state
                        .set_error_message
                        .set(Some(format!("部分删除失败: {}", errors.join(", "))));
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
                    "导入"
                </button>
                <button class="btn btn--secondary btn--sm" on:click=move |_| on_export()>
                    "导出"
                </button>
                <button class="btn btn--primary" on:click=move |_| on_add_click()>
                    "+ 添加别名"
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

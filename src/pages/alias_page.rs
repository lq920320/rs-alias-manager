/// 别名管理页面。
///
/// 提供查看、添加、编辑和删除 Shell 别名的主界面。
/// 包含搜索过滤和多选操作功能。
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::{Event, FileReader, HtmlInputElement};

use crate::components::alias_form::AliasForm;
use crate::components::alias_list::AliasList;
use crate::components::search_bar::SearchBar;
use crate::i18n::t;
use crate::state::app_state::AppState;
use crate::state::app_state::Alias;
use crate::utils::trigger_download;

/// 变更成功后，若开启 instant_apply 则对配置文件执行 source（仅对新终端生效）。
fn trigger_auto_source(state: AppState) {
    let settings = state.settings.get();
    if settings.instant_apply {
        let shell = settings.shell_type.to_string();
        spawn_local(async move {
            if let Err(e) = crate::api::commands::auto_source(shell).await {
                log::warn!("auto_source failed: {}", e);
            }
        });
    }
}

/// 重新加载别名列表（不阻塞 UI）。
fn reload_aliases(state: AppState) {
    spawn_local(async move {
        match crate::api::commands::list_aliases().await {
            Ok(aliases) => state.set_aliases.set(aliases),
            Err(e) => state.set_error_message.set(Some(e.display())),
        }
    });
}

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

    // 单条别名冲突（覆盖/保留）对话框状态
    let (show_conflict, set_show_conflict) = signal(false);
    let (conflict_name, set_conflict_name) = signal(String::new());
    let (pending_overwrite, set_pending_overwrite) =
        signal(None::<(String, String, String, Vec<String>)>);

    // 批量导入冲突（覆盖全部）对话框状态
    let (show_import_conflict, set_show_import_conflict) = signal(false);
    let (import_skipped_count, set_import_skipped_count) = signal(0usize);
    let (pending_import_overwrite, set_pending_import_overwrite) =
        signal(None::<Vec<Alias>>);

    // 隐藏的文件选择器，用于导入 JSON 文件
    let file_input: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_import = {
        let state = state;
        move || {
            if let Some(input) = file_input.get() {
                let _ = input.click();
            }
            let _ = state;
        }
    };

    let on_file_change = {
        let state = state;
        move |ev: Event| {
            let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
            let file = match input.files().and_then(|f| f.get(0)) {
                Some(f) => f,
                None => return,
            };
            let state = state;
            let skipped_setter = set_import_skipped_count;
            let pending_setter = set_pending_import_overwrite;
            let show_setter = set_show_import_conflict;
            match FileReader::new() {
                Ok(reader) => {
                    let reader_clone = reader.clone();
                    let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: Event| {
                        if let Ok(text) = reader_clone.result() {
                            if let Some(s) = text.as_string() {
                                let state = state;
                                spawn_local(async move {
                                    match serde_json::from_str::<Vec<Alias>>(&s) {
                                        Ok(aliases) => {
                                            match crate::api::commands::batch_add_aliases(aliases.clone()).await {
                                                Ok(result) => {
                                                    if result.skipped_count > 0 {
                                                        skipped_setter.set(result.skipped_count);
                                                        pending_setter.set(Some(aliases));
                                                        show_setter.set(true);
                                                    } else if !result.errors.is_empty() {
                                                        state.set_error_message.set(Some(
                                                            t("alias.import_partial")
                                                                .replace("{}", &result.success_count.to_string())
                                                                .replace("{}", &result.errors.join(", ")),
                                                        ));
                                                    } else if result.success_count > 0 {
                                                        state.set_success_message.set(Some(
                                                            t("alias.import_success")
                                                                .replace("{}", &result.success_count.to_string()),
                                                        ));
                                                    }
                                                    reload_aliases(state);
                                                    trigger_auto_source(state);
                                                },
                                                Err(e) => state.set_error_message.set(Some(e.display())),
                                            }
                                        },
                                        Err(e) => {
                                            state.set_error_message.set(Some(
                                                t("alias.json_parse_error").replace("{}", &e.to_string())
                                            ));
                                        },
                                    }
                                });
                            }
                        }
                    }) as Box<dyn Fn(Event)>);
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    let _ = reader.read_as_text(&file);
                },
                Err(_) => state.set_error_message.set(Some("Failed to read file".to_string())),
            }
        }
    };

    let on_conflict_overwrite = {
        let state = state;
        move |_: ()| {
            let state = state;
            set_show_conflict.set(false);
            if let Some((_old, name, command, tags)) = pending_overwrite.get() {
                spawn_local(async move {
                    match crate::api::commands::update_alias(name.clone(), name, command, tags).await {
                        Ok(()) => {
                            set_show_form.set(false);
                            set_editing_alias.set(None);
                            reload_aliases(state);
                            trigger_auto_source(state);
                        },
                        Err(e) => state.set_error_message.set(Some(e.display())),
                    }
                });
            }
        }
    };

    let on_conflict_keep = move |_: ()| {
        set_show_conflict.set(false);
    };

    let on_import_overwrite_all = {
        let state = state;
        move |_: ()| {
            let state = state;
            set_show_import_conflict.set(false);
            if let Some(aliases) = pending_import_overwrite.get() {
                spawn_local(async move {
                    for a in aliases {
                        let _ = crate::api::commands::update_alias(
                            a.name.clone(),
                            a.name.clone(),
                            a.command.clone(),
                            a.tags.clone(),
                        ).await;
                    }
                    reload_aliases(state);
                    trigger_auto_source(state);
                });
            }
        }
    };

    let on_import_conflict_cancel = move |_: ()| {
        set_show_import_conflict.set(false);
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
                        reload_aliases(state);
                        trigger_auto_source(state);
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e.display()));
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
            let add_name = name.clone();
            let add_command = command.clone();
            let add_tags = tags.clone();
            spawn_local(async move {
                state.set_loading.set(true);
                state.set_error_message.set(None);
                let result = match old_name {
                    Some(old) => crate::api::commands::update_alias(old, name.clone(), command.clone(), tags.clone()).await,
                    None => crate::api::commands::add_alias(add_name, add_command, add_tags).await,
                };
                match result {
                    Ok(()) => {
                        set_show_form.set(false);
                        set_editing_alias.set(None);
                        reload_aliases(state);
                        trigger_auto_source(state);
                    },
                    // 重名冲突：弹「覆盖/保留」对话框，覆盖则走 update_alias。
                    // 按后端结构化错误码分支，不再依赖文案匹配。
                    Err(e) if e.code == "alias_conflict" => {
                        let cname = e.detail.unwrap_or_else(|| name.clone());
                        set_conflict_name.set(cname);
                        set_pending_overwrite.set(Some((
                            name.clone(),
                            name.clone(),
                            command.clone(),
                            tags.clone(),
                        )));
                        set_show_conflict.set(true);
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e.display()));
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
                let deleted_count = selected.len();
                match crate::api::commands::batch_delete_aliases(selected.to_vec()).await {
                    Ok(result) => {
                        if !result.errors.is_empty() {
                            state.set_error_message.set(Some(
                                t("alias.delete_partial").replace("{}", &result.errors.join(", "))
                            ));
                        } else {
                            state.set_success_message.set(Some(
                                t("alias.delete_success").replace("{}", &deleted_count.to_string())
                            ));
                        }
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e.display()));
                    },
                }
                state.set_selected_aliases.set(Vec::new());
                reload_aliases(state);
                trigger_auto_source(state);
                state.set_loading.set(false);
            });
        }
    };

    view! {
        // 隐藏的文件选择器，供「从文件导入」使用
        <input
            type="file"
            accept="application/json,.json"
            style="display: none"
            node_ref=file_input
            on:change=on_file_change
        />

        <div class="app-header">
            <div class="search-bar">
                <SearchBar />
            </div>
            <div class="app-header__actions">
                <button class="btn btn--secondary btn--sm" on:click=move |_| on_import()>
                    {move || t("alias.import_file")}
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
                    let msg = state.success_message.get();
                    if let Some(m) = msg {
                        view! {
                            <div class="alert alert--success mb-lg">
                                {m}
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

        {
            move || {
                if show_conflict.get() {
                    let cname = conflict_name.get();
                    view! {
                        <div class="modal-backdrop">
                            <div class="modal">
                                <div class="modal__header">
                                    <span class="modal__title">{move || t("alias.conflict_title")}</span>
                                    <button class="btn btn--icon" on:click=move |_| on_conflict_keep(())>
                                        &times;
                                    </button>
                                </div>
                                <div class="modal__body">
                                    {move || t("alias.conflict_msg").replace("{}", &cname)}
                                </div>
                                <div class="modal__footer">
                                    <button class="btn btn--secondary" on:click=move |_| on_conflict_keep(())>
                                        {move || t("alias.conflict_keep")}
                                    </button>
                                    <button class="btn btn--primary" on:click=move |_| on_conflict_overwrite(())>
                                        {move || t("alias.conflict_overwrite")}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }
        }

        {
            move || {
                if show_import_conflict.get() {
                    let count = import_skipped_count.get();
                    view! {
                        <div class="modal-backdrop">
                            <div class="modal">
                                <div class="modal__header">
                                    <span class="modal__title">{move || t("alias.import_conflict_title")}</span>
                                    <button class="btn btn--icon" on:click=move |_| on_import_conflict_cancel(())>
                                        &times;
                                    </button>
                                </div>
                                <div class="modal__body">
                                    {move || t("alias.import_conflict_msg").replace("{}", &count.to_string())}
                                </div>
                                <div class="modal__footer">
                                    <button class="btn btn--secondary" on:click=move |_| on_import_conflict_cancel(())>
                                        {move || t("alias.conflict_keep")}
                                    </button>
                                    <button class="btn btn--primary" on:click=move |_| on_import_overwrite_all(())>
                                        {move || t("alias.import_overwrite_all")}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }
        }
    }
}

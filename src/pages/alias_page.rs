/// 别名管理页面。
///
/// 提供查看、添加、编辑和删除 Shell 别名的主界面。
/// 包含搜索过滤、多选操作、导入预览与配置文件查看功能。
use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::{Event, FileReader, HtmlInputElement};

use crate::components::alias_form::AliasForm;
use crate::components::alias_list::AliasList;
use crate::components::config_viewer::ConfigViewer;
use crate::components::search_bar::SearchBar;
use crate::i18n::t;
use crate::state::app_state::Alias;
use crate::state::app_state::AppState;
use crate::utils::trigger_download;

/// 导入回调的共享槽位：回调触发后置 None，令 Closure 自行释放。
type ImportCbSlot = Rc<RefCell<Option<wasm_bindgen::closure::Closure<dyn Fn(Event)>>>>;

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

/// 导入条目状态（预览对话框）。
#[derive(Debug, Clone, Copy, PartialEq)]
enum ImportStatus {
    /// 新别名，当前配置中不存在。
    New,
    /// 同名已存在且内容不同，用户可勾选覆盖。
    Conflict,
    /// 同名已存在且内容完全一致，无需处理。
    Unchanged,
    /// 名称或命令未通过校验，不可导入。
    Invalid,
}

/// 导入预览中的一行。
#[derive(Clone)]
struct ImportRow {
    alias: Alias,
    status: ImportStatus,
    /// 校验失败原因对应的 i18n 键（渲染时本地化）。
    reason_key: Option<&'static str>,
    /// 用户对冲突条目的覆盖选择。
    overwrite: bool,
}

/// 校验名称并返回未本地化的原因键（供预览构建在非响应式上下文中使用）。
fn name_invalid_reason(name: &str) -> Option<&'static str> {
    if name.is_empty() {
        Some("validate.name_empty")
    } else if name.starts_with('-') {
        Some("validate.name_hyphen")
    } else if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Some("validate.name_chars")
    } else {
        None
    }
}

/// 构建导入预览条目：文件内按名称去重（后出现的生效），
/// 再对照现有别名分为新增 / 冲突 / 无变化 / 无效四类。
///
/// 返回（预览行，文件内被合并的重复条目数）。
fn build_import_preview(parsed: Vec<Alias>, existing: &[Alias]) -> (Vec<ImportRow>, usize) {
    let mut merged: Vec<Alias> = Vec::new();
    let mut dup_merged = 0usize;
    for a in parsed {
        if a.name.is_empty() {
            // 无名条目不参与去重，交由校验标记为无效
            merged.push(a);
            continue;
        }
        match merged.iter().position(|m| m.name == a.name) {
            Some(pos) => {
                merged[pos] = a;
                dup_merged += 1;
            },
            None => merged.push(a),
        }
    }

    let rows = merged
        .into_iter()
        .map(|a| {
            if let Some(key) = name_invalid_reason(&a.name) {
                ImportRow {
                    alias: a,
                    status: ImportStatus::Invalid,
                    reason_key: Some(key),
                    overwrite: false,
                }
            } else if a.command.trim().is_empty() {
                ImportRow {
                    alias: a,
                    status: ImportStatus::Invalid,
                    reason_key: Some("validate.command_empty"),
                    overwrite: false,
                }
            } else {
                match existing.iter().find(|e| e.name == a.name) {
                    None => ImportRow {
                        alias: a,
                        status: ImportStatus::New,
                        reason_key: None,
                        overwrite: false,
                    },
                    Some(cur) if cur.command == a.command && cur.tags == a.tags => ImportRow {
                        alias: a,
                        status: ImportStatus::Unchanged,
                        reason_key: None,
                        overwrite: false,
                    },
                    Some(_) => ImportRow {
                        alias: a,
                        status: ImportStatus::Conflict,
                        reason_key: None,
                        overwrite: false,
                    },
                }
            }
        })
        .collect();
    (rows, dup_merged)
}

/// 别名管理页面组件。
#[component]
pub fn AliasPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");
    let (show_form, set_show_form) = signal(false);
    let (editing_alias, set_editing_alias) = signal(None::<(String, String, Vec<String>)>);
    let (show_config_viewer, set_show_config_viewer) = signal(false);

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

    // 导入预览对话框状态：None 表示未打开
    let (import_rows, set_import_rows) = signal(None::<Vec<ImportRow>>);
    let (import_dup_merged, set_import_dup_merged) = signal(0usize);

    // 隐藏的文件选择器，用于导入 JSON 文件
    let file_input: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_import = {
        move || {
            if let Some(input) = file_input.get() {
                input.click();
            }
            let _ = state;
        }
    };

    let on_file_change = {
        move |ev: Event| {
            let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
            let file = match input.files().and_then(|f| f.get(0)) {
                Some(f) => f,
                None => return,
            };
            // 清空以便再次选择同一文件仍能触发 change
            input.set_value("");
            let set_rows = set_import_rows;
            let set_dup = set_import_dup_merged;
            match FileReader::new() {
                Ok(reader) => {
                    let reader_for_cb = reader.clone();
                    // 回调触发后自行释放，避免每次导入泄漏一个 Closure
                    let slot: ImportCbSlot = Rc::new(RefCell::new(None));
                    let onload = wasm_bindgen::closure::Closure::wrap(Box::new({
                        let slot = slot.clone();
                        move |_e: Event| {
                            *slot.borrow_mut() = None;
                            if let Ok(text) = reader_for_cb.result() {
                                if let Some(s) = text.as_string() {
                                    let existing = state.aliases.get().to_vec();
                                    match serde_json::from_str::<Vec<Alias>>(&s) {
                                        Ok(parsed) => {
                                            let (rows, dup) =
                                                build_import_preview(parsed, &existing);
                                            set_dup.set(dup);
                                            set_rows.set(Some(rows));
                                        },
                                        Err(e) => {
                                            state.set_error_message.set(Some(
                                                t("alias.json_parse_error")
                                                    .replace("{}", &e.to_string()),
                                            ));
                                        },
                                    }
                                }
                            }
                        }
                    })
                        as Box<dyn Fn(Event)>);
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    *slot.borrow_mut() = Some(onload);
                    let _ = reader.read_as_text(&file);
                },
                Err(_) => state
                    .set_error_message
                    .set(Some("Failed to read file".to_string())),
            }
        }
    };

    let on_conflict_overwrite = {
        move |_: ()| {
            set_show_conflict.set(false);
            if let Some((_old, name, command, tags)) = pending_overwrite.get() {
                spawn_local(async move {
                    match crate::api::commands::update_alias(name.clone(), name, command, tags)
                        .await
                    {
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

    // === 导入预览对话框操作 ===

    let on_preview_cancel = move |_: ()| {
        set_import_rows.set(None);
    };

    // 切换某条冲突条目的覆盖选择。
    let on_toggle_overwrite = move |idx: usize| {
        set_import_rows.update(|opt| {
            if let Some(rows) = opt {
                if let Some(row) = rows.get_mut(idx) {
                    row.overwrite = !row.overwrite;
                }
            }
        });
    };

    // 勾选全部冲突条目的覆盖。
    let on_preview_overwrite_all = move |_: ()| {
        set_import_rows.update(|opt| {
            if let Some(rows) = opt {
                for row in rows.iter_mut() {
                    if row.status == ImportStatus::Conflict {
                        row.overwrite = true;
                    }
                }
            }
        });
    };

    // 确认导入：新增条目走批量添加，勾选的冲突条目走批量覆盖，各只写一次配置文件。
    let on_preview_confirm = {
        move |_: ()| {
            let rows = match import_rows.get() {
                Some(r) => r,
                None => return,
            };
            let to_add: Vec<Alias> = rows
                .iter()
                .filter(|r| r.status == ImportStatus::New)
                .map(|r| r.alias.clone())
                .collect();
            let to_update: Vec<Alias> = rows
                .iter()
                .filter(|r| r.status == ImportStatus::Conflict && r.overwrite)
                .map(|r| r.alias.clone())
                .collect();
            if to_add.is_empty() && to_update.is_empty() {
                return;
            }
            set_import_rows.set(None);
            spawn_local(async move {
                state.set_loading.set(true);
                let mut success = 0usize;
                let mut errors: Vec<String> = Vec::new();
                let mut fatal = false;
                if !to_add.is_empty() {
                    match crate::api::commands::batch_add_aliases(to_add).await {
                        Ok(r) => {
                            success += r.success_count;
                            errors.extend(r.errors);
                            if r.skipped_count > 0 {
                                // 预览后配置被外部修改导致重名，按跳过处理
                                log::info!(
                                    "import: {} entries skipped (already exist)",
                                    r.skipped_count
                                );
                            }
                        },
                        Err(e) => {
                            state.set_error_message.set(Some(e.display()));
                            fatal = true;
                        },
                    }
                }
                if !fatal && !to_update.is_empty() {
                    match crate::api::commands::batch_update_aliases(to_update).await {
                        Ok(r) => {
                            success += r.success_count;
                            errors.extend(r.errors);
                        },
                        Err(e) => {
                            state.set_error_message.set(Some(e.display()));
                        },
                    }
                }
                if errors.is_empty() {
                    if success > 0 {
                        state.set_success_message.set(Some(
                            t("alias.import_success").replace("{}", &success.to_string()),
                        ));
                    }
                } else {
                    state.set_error_message.set(Some(
                        t("alias.import_partial")
                            .replacen("{}", &success.to_string(), 1)
                            .replace("{}", &errors.join(", ")),
                    ));
                }
                reload_aliases(state);
                trigger_auto_source(state);
                state.set_loading.set(false);
            });
        }
    };

    let on_edit = move |(name, command, tags): (String, String, Vec<String>)| {
        set_editing_alias.set(Some((name, command, tags)));
        set_show_form.set(true);
    };

    let on_delete = {
        move |name: String| {
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
        move |(old_name, name, command, tags): (Option<String>, String, String, Vec<String>)| {
            let add_name = name.clone();
            let add_command = command.clone();
            let add_tags = tags.clone();
            spawn_local(async move {
                state.set_loading.set(true);
                state.set_error_message.set(None);
                let result = match old_name {
                    Some(old) => {
                        crate::api::commands::update_alias(
                            old,
                            name.clone(),
                            command.clone(),
                            tags.clone(),
                        )
                        .await
                    },
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
        move || {
            let aliases = state.aliases.get();
            let json = serde_json::to_string_pretty(&*aliases).unwrap_or_else(|_| "[]".to_string());
            trigger_download("aliases.json", &json);
        }
    };

    let on_delete_selected = {
        move |_: ()| {
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
                                t("alias.delete_partial").replace("{}", &result.errors.join(", ")),
                            ));
                        } else {
                            state.set_success_message.set(Some(
                                t("alias.delete_success").replace("{}", &deleted_count.to_string()),
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
                <button class="btn btn--secondary btn--sm" on:click=move |_| set_show_config_viewer.set(true)>
                    {move || t("alias.view_config")}
                </button>
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
                let rows = match import_rows.get() {
                    Some(r) => r,
                    None => return view! { <div></div> }.into_any(),
                };
                let dup = import_dup_merged.get();
                let n_new = rows.iter().filter(|r| r.status == ImportStatus::New).count();
                let n_conflict = rows.iter().filter(|r| r.status == ImportStatus::Conflict).count();
                let n_unchanged = rows.iter().filter(|r| r.status == ImportStatus::Unchanged).count();
                let n_invalid = rows.iter().filter(|r| r.status == ImportStatus::Invalid).count();
                let n_overwrite = rows
                    .iter()
                    .filter(|r| r.status == ImportStatus::Conflict && r.overwrite)
                    .count();
                let n_effective = n_new + n_overwrite;
                let summary = t("alias.import_preview_summary")
                    .replacen("{}", &n_new.to_string(), 1)
                    .replacen("{}", &n_conflict.to_string(), 1)
                    .replacen("{}", &n_unchanged.to_string(), 1)
                    .replacen("{}", &n_invalid.to_string(), 1);
                view! {
                    <div class="modal-overlay" on:click=move |_| on_preview_cancel(())>
                        <div class="modal modal--wide" on:click=|e| e.stop_propagation()>
                            <div class="modal__header">
                                <h2 class="modal__title">{move || t("alias.import_preview_title")}</h2>
                                <button class="modal__close" on:click=move |_| on_preview_cancel(())>
                                    "✕"
                                </button>
                            </div>
                            <div class="modal__body">
                                <div class="import-preview__summary">{summary}</div>
                                {
                                    if dup > 0 {
                                        view! {
                                            <div class="import-preview__note">
                                                {t("alias.import_dedup_note").replace("{}", &dup.to_string())}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }
                                <table class="import-preview-table">
                                    <thead>
                                        <tr>
                                            <th></th>
                                            <th>{t("form.name_label")}</th>
                                            <th>{t("form.command_label")}</th>
                                            <th>{t("alias.import_overwrite_col")}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {rows.iter().enumerate().map(|(idx, row)| {
                                            let (status_class, status_label) = match row.status {
                                                ImportStatus::New => ("import-status--new", t("alias.import_status_new")),
                                                ImportStatus::Conflict => ("import-status--conflict", t("alias.import_status_conflict")),
                                                ImportStatus::Unchanged => ("import-status--unchanged", t("alias.import_status_unchanged")),
                                                ImportStatus::Invalid => ("import-status--invalid", t("alias.import_status_invalid")),
                                            };
                                            let display_name = if row.alias.name.is_empty() {
                                                "-".to_string()
                                            } else {
                                                row.alias.name.clone()
                                            };
                                            let reason = row.reason_key.map(t);
                                            let is_conflict = row.status == ImportStatus::Conflict;
                                            let checked = row.overwrite;
                                            view! {
                                                <tr>
                                                    <td>
                                                        <span class=format!("import-status {}", status_class)>
                                                            {status_label}
                                                        </span>
                                                    </td>
                                                    <td class="import-preview-table__name">
                                                        {display_name}
                                                        {reason.map(|r| view! {
                                                            <div class="import-preview-table__reason">{r}</div>
                                                        })}
                                                    </td>
                                                    <td class="import-preview-table__command">
                                                        {row.alias.command.clone()}
                                                    </td>
                                                    <td class="import-preview-table__overwrite">
                                                        {
                                                            if is_conflict {
                                                                view! {
                                                                    <input
                                                                        type="checkbox"
                                                                        checked=checked
                                                                        on:change=move |_| on_toggle_overwrite(idx)
                                                                    />
                                                                }.into_any()
                                                            } else {
                                                                view! { <span></span> }.into_any()
                                                            }
                                                        }
                                                    </td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="modal__footer">
                                <button class="btn btn--secondary" on:click=move |_| on_preview_cancel(())>
                                    {move || t("form.cancel")}
                                </button>
                                {
                                    if n_conflict > 0 {
                                        view! {
                                            <button class="btn btn--secondary" on:click=move |_| on_preview_overwrite_all(())>
                                                {move || t("alias.import_overwrite_all")}
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }
                                <button
                                    class="btn btn--primary"
                                    disabled=n_effective == 0
                                    on:click=move |_| on_preview_confirm(())
                                >
                                    {move || format!("{} ({})", t("alias.import_confirm"), n_effective)}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            }
        }

        {
            move || {
                if show_config_viewer.get() {
                    view! {
                        <ConfigViewer on_close=Callback::new(move |_| set_show_config_viewer.set(false)) />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alias(name: &str, command: &str, tags: &[&str]) -> Alias {
        Alias {
            name: name.to_string(),
            command: command.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
        }
    }

    #[test]
    fn test_preview_classifies_new_unchanged_conflict() {
        let existing = vec![
            alias("gs", "git status", &[]),
            alias("ll", "ls -la", &["file"]),
        ];
        let parsed = vec![
            alias("gp", "git push", &[]),      // 新增
            alias("gs", "git status", &[]),    // 完全一致
            alias("ll", "ls -lah", &["file"]), // 命令不同 → 冲突
        ];
        let (rows, dup) = build_import_preview(parsed, &existing);
        assert_eq!(dup, 0);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].status, ImportStatus::New);
        assert_eq!(rows[1].status, ImportStatus::Unchanged);
        assert_eq!(rows[2].status, ImportStatus::Conflict);
        assert!(!rows[2].overwrite);
    }

    #[test]
    fn test_preview_marks_invalid_entries() {
        let parsed = vec![
            alias("bad name", "echo hi", &[]),
            alias("", "echo hi", &[]),
            alias("ok", "   ", &[]), // 空白命令
        ];
        let (rows, _) = build_import_preview(parsed, &[]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].status, ImportStatus::Invalid);
        assert_eq!(rows[0].reason_key, Some("validate.name_chars"));
        assert_eq!(rows[1].status, ImportStatus::Invalid);
        assert_eq!(rows[1].reason_key, Some("validate.name_empty"));
        assert_eq!(rows[2].status, ImportStatus::Invalid);
        assert_eq!(rows[2].reason_key, Some("validate.command_empty"));
    }

    #[test]
    fn test_preview_merges_duplicates_last_wins() {
        let parsed = vec![
            alias("gs", "git status", &[]),
            alias("gs", "git status --short", &["git"]),
        ];
        let (rows, dup) = build_import_preview(parsed, &[]);
        assert_eq!(dup, 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].alias.command, "git status --short");
        assert_eq!(rows[0].alias.tags, vec!["git".to_string()]);
    }

    #[test]
    fn test_preview_tags_participate_in_conflict_detection() {
        let existing = vec![alias("gs", "git status", &[])];
        let parsed = vec![alias("gs", "git status", &["git"])];
        let (rows, _) = build_import_preview(parsed, &existing);
        assert_eq!(rows[0].status, ImportStatus::Conflict);
    }
}

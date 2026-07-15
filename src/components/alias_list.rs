/// 别名列表组件。
///
/// 展示别名列表，支持：
/// - 通过复选框多选
/// - 每个别名的编辑和删除操作
/// - 批量删除选中的别名
/// - 空状态展示
use leptos::prelude::*;

use crate::i18n::t;
use crate::state::app_state::AppState;

/// 别名列表组件。
///
/// # 属性
/// * `on_edit` - 用户点击编辑别名时的回调
/// * `on_delete` - 用户点击删除别名时的回调
/// * `on_delete_selected` - 用户点击删除选中项时的回调
#[component]
pub fn AliasList(
    on_edit: Callback<(String, String, Vec<String>)>,
    on_delete: Callback<String>,
    on_delete_selected: Callback<()>,
) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");

    let toggle_select = move |name: String| {
        let mut selected = state.selected_aliases.get().to_vec();
        if let Some(pos) = selected.iter().position(|n| n == &name) {
            selected.remove(pos);
        } else {
            selected.push(name);
        }
        state.set_selected_aliases.set(selected);
    };

    let toggle_select_all = move || {
        let aliases = state.filtered_aliases();
        let selected = state.selected_aliases.get();
        let all_selected = aliases.iter().all(|a| selected.contains(&a.name));
        if all_selected {
            state.set_selected_aliases.set(Vec::new());
        } else {
            state.set_selected_aliases.set(aliases.iter().map(|a| a.name.clone()).collect());
        }
    };

    view! {
        <div class="alias-list">
            <div class="alias-list__header">
                <div class="alias-list__count">
                    {
                        move || {
                            let aliases = state.filtered_aliases();
                            let total = state.aliases.get().len();
                            let filtered = aliases.len();
                            if state.search_query.get().is_empty() {
                                t("alias.count").replace("{}", &total.to_string())
                            } else {
                                t("alias.count_filtered")
                                    .replacen("{}", &filtered.to_string(), 1)
                                    .replacen("{}", &total.to_string(), 1)
                            }
                        }
                    }
                </div>
                <div class="alias-list__actions">
                    {
                        move || {
                            let selected = state.selected_aliases.get();
                            if !selected.is_empty() {
                                view! {
                                    <button
                                        class="btn btn--danger btn--sm"
                                        on:click=move |_| on_delete_selected.run(())
                                    >
                                        { t("alias.delete_selected").replace("{}", &selected.len().to_string()) }
                                    </button>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }
                    }
                </div>
            </div>

            {
                move || {
                    let aliases = state.filtered_aliases();
                    let selected = state.selected_aliases.get();
                    let all_selected = !aliases.is_empty() && aliases.iter().all(|a| selected.contains(&a.name));

                    if aliases.is_empty() {
                        let query = state.search_query.get();
                        if query.is_empty() {
                            view! {
                                <div class="empty-state">
                                    <div class="empty-state__icon">"📋"</div>
                                    <div class="empty-state__title">{t("alias.empty_title")}</div>
                                    <div class="empty-state__description">
                                        {t("alias.empty_desc")}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="empty-state">
                                    <div class="empty-state__icon">"🔍"</div>
                                    <div class="empty-state__title">{t("alias.search_empty_title")}</div>
                                    <div class="empty-state__description">
                                        { t("alias.search_empty_desc").replace("{}", &query) }
                                    </div>
                                </div>
                            }.into_any()
                        }
                    } else {
                        view! {
                            <div>
                                <div class="alias-item" style="cursor:default;border:none;padding:8px 16px;background:transparent">
                                    <input
                                        type="checkbox"
                                        class="alias-item__checkbox"
                                        checked=all_selected
                                        on:change=move |_| toggle_select_all()
                                    />
                                    <div style="flex:1;font-size:12px;color:var(--text-muted)">
                                        {t("alias.select_all")}
                                    </div>
                                </div>
                                <For
                                    each=move || state.filtered_aliases()
                                    key=|alias| alias.name.clone()
                                    children=move |alias| {
                                        let alias_name = alias.name.clone();
                                        let alias_command = alias.command.clone();
                                        let alias_tags = alias.tags.clone();
                                        let is_selected = selected.contains(&alias.name);
                                        let name_for_edit = alias.name.clone();
                                        let cmd_for_edit = alias.command.clone();
                                        let tags_for_edit = alias.tags.clone();
                                        let name_for_delete = alias.name.clone();
                                        let name_for_toggle = alias.name.clone();

                                        view! {
                                            <div class=format!("alias-item{}", if is_selected { " alias-item--selected" } else { "" })>
                                                <input
                                                    type="checkbox"
                                                    class="alias-item__checkbox"
                                                    checked=is_selected
                                                    on:change=move |_| toggle_select(name_for_toggle.clone())
                                                />
                                                <div class="alias-item__content">
                                                    <div class="alias-item__name">{ alias_name }</div>
                                                    <div class="alias-item__command">{ alias_command }</div>
                                                </div>
                                                {
                                                    if alias_tags.is_empty() {
                                                        view! { <div></div> }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="alias-item__tags">
                                                                {alias_tags.iter().enumerate().map(|(i, tag)| {
                                                                    let color_class = match i % 6 {
                                                                        0 => "tag--blue",
                                                                        1 => "tag--green",
                                                                        2 => "tag--purple",
                                                                        3 => "tag--orange",
                                                                        4 => "tag--pink",
                                                                        _ => "tag--cyan",
                                                                    };
                                                                    view! {
                                                                        <span class=format!("tag {}", color_class)>{ tag.clone() }</span>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }
                                                <div class="alias-item__actions">
                                                    <button
                                                        class="btn btn--ghost btn--sm"
                                                        on:click=move |_| on_edit.run((name_for_edit.clone(), cmd_for_edit.clone(), tags_for_edit.clone()))
                                                    >
                                                        {t("alias.edit")}
                                                    </button>
                                                    <button
                                                        class="btn btn--ghost btn--sm text-danger"
                                                        on:click=move |_| on_delete.run(name_for_delete.clone())
                                                    >
                                                        {t("alias.delete")}
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_any()
                    }
                }
            }
        </div>
    }
}

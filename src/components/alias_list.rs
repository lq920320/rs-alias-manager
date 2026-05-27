/// 别名列表组件。
///
/// 展示别名列表，支持：
/// - 通过复选框多选
/// - 每个别名的编辑和删除操作
/// - 批量删除选中的别名
/// - 空状态展示
use leptos::prelude::*;

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
                                format!("共 {} 个别名", total)
                            } else {
                                format!("找到 {} / {} 个别名", filtered, total)
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
                                        { format!("删除选中 ({})", selected.len()) }
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
                                    <div class="empty-state__title">"还没有别名"</div>
                                    <div class="empty-state__description">
                                        "点击右上角的「添加别名」按钮创建你的第一个别名"
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="empty-state">
                                    <div class="empty-state__icon">"🔍"</div>
                                    <div class="empty-state__title">"没有找到匹配的别名"</div>
                                    <div class="empty-state__description">
                                        { format!("没有与「{}」匹配的别名", query) }
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
                                        "全选"
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
                                                    {
                                                        if alias_tags.is_empty() {
                                                            view! { <div></div> }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="alias-item__tags">
                                                                    {alias_tags.iter().map(|tag| view! {
                                                                        <span class="tag">{ tag.clone() }</span>
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }
                                                </div>
                                                <div class="alias-item__actions">
                                                    <button
                                                        class="btn btn--ghost btn--sm"
                                                        on:click=move |_| on_edit.run((name_for_edit.clone(), cmd_for_edit.clone(), tags_for_edit.clone()))
                                                    >
                                                        "编辑"
                                                    </button>
                                                    <button
                                                        class="btn btn--ghost btn--sm text-danger"
                                                        on:click=move |_| on_delete.run(name_for_delete.clone())
                                                    >
                                                        "删除"
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

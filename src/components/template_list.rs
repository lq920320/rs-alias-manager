/// 模板列表组件。
///
/// 展示带有选择复选框的模板列表。
use leptos::prelude::*;

use crate::i18n::t;
use crate::state::app_state::{Template, TemplateCategory};

/// 模板列表组件。
///
/// # 属性
/// * `templates` - 包含要展示的模板列表的信号
/// * `selected` - 包含已选中模板名称列表的信号
/// * `on_toggle` - 切换模板复选框时的回调
/// * `on_delete` - 删除用户自定义模板时的回调（仅 Custom 分类显示）
#[component]
pub fn TemplateList(
    templates: ReadSignal<Vec<Template>>,
    selected: ReadSignal<Vec<String>>,
    on_toggle: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="template-list">
            {
                move || {
                    let templates = templates.get();
                    if templates.is_empty() {
                        view! {
                            <div class="empty-state">
                                <div class="empty-state__icon">"📦"</div>
                                <div class="empty-state__title">{t("template.empty_title")}</div>
                                <div class="empty-state__description">
                                    {t("template.empty_desc")}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <For
                                each=move || templates.clone()
                                key=|t| t.name.clone()
                                children=move |template| {
                                    let _name = template.name.clone();
                                    let is_selected = selected.get().contains(&template.name);
                                    let toggle_name = template.name.clone();
                                    let is_custom = template.category == TemplateCategory::Custom;
                                    let delete_name = template.name.clone();

                                    view! {
                                        <div class="template-item">
                                            <input
                                                type="checkbox"
                                                class="template-item__checkbox"
                                                checked=is_selected
                                                on:change=move |_| on_toggle.run(toggle_name.clone())
                                            />
                                            <div class="template-item__content">
                                                <div class="template-item__name">{ template.name }</div>
                                                <div class="template-item__command">{ template.command }</div>
                                                <div class="template-item__description">{ template.description }</div>
                                            </div>
                                            {
                                                if is_custom {
                                                    view! {
                                                        <button
                                                            class="btn btn--ghost btn--sm text-danger"
                                                            on:click=move |_| on_delete.run(delete_name.clone())
                                                        >
                                                            {t("template.delete")}
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }
                                            }
                                        </div>
                                    }
                                }
                            />
                        }.into_any()
                    }
                }
            }
        </div>
    }
}

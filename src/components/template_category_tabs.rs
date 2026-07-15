/// 模板分类标签页组件。
///
/// 展示用于按分类过滤模板的水平标签栏。
use leptos::prelude::*;

use crate::i18n::t;
use crate::state::app_state::TemplateCategory;

/// 模板分类标签页组件。
///
/// # 属性
/// * `active_category` - 当前选中分类的信号（None = 全部）
/// * `on_select` - 点击分类标签时的回调
#[component]
pub fn TemplateCategoryTabs(
    active_category: ReadSignal<Option<TemplateCategory>>,
    on_select: Callback<Option<TemplateCategory>>,
) -> impl IntoView {
    view! {
        <div class="template-category-tabs">
            <button
                class=move || format!(
                    "template-category-tabs__tab{}",
                    if active_category.get().is_none() { " template-category-tabs__tab--active" } else { "" }
                )
                on:click=move |_| on_select.run(None)
            >
                {move || t("template.all")}
            </button>
            {
                TemplateCategory::all().into_iter().map(|cat| {
                    let cat_label = cat.label().to_string();
                    view! {
                        <button
                            class=move || format!(
                                "template-category-tabs__tab{}",
                                if active_category.get() == Some(cat) { " template-category-tabs__tab--active" } else { "" }
                            )
                            on:click=move |_| on_select.run(Some(cat))
                        >
                            { cat_label.clone() }
                        </button>
                    }
                }).collect::<Vec<_>>()
            }
        </div>
    }
}

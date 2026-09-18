/// 模板库页面。
///
/// 按分类展示内置别名模板，允许用户浏览并导入选中的模板。
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::template_category_tabs::TemplateCategoryTabs;
use crate::components::template_list::TemplateList;
use crate::i18n::t;
use crate::state::app_state::{AppState, TemplateCategory};
use crate::utils::set_timeout;

/// 模板库页面组件。
#[component]
pub fn TemplatePage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");
    let (active_category, set_active_category) = signal(None::<TemplateCategory>);
    let (selected_templates, set_selected_templates) = signal(Vec::<String>::new());

    // 挂载时加载模板
    let (templates, set_templates) = signal(Vec::<crate::state::app_state::Template>::new());

    Effect::new(move || {
        let category = active_category.get();
        let category_str = category.map(|c| c.to_string());
        spawn_local(async move {
            match crate::api::commands::list_templates(category_str).await {
                Ok(t) => set_templates.set(t),
                Err(e) => {
                    log::warn!("Failed to load templates: {}", e);
                },
            }
        });
    });

    let on_import = {
        move || {
            let names = selected_templates.get();
            if names.is_empty() {
                return;
            }
            let names_clone = names.clone();
            spawn_local(async move {
                state.set_loading.set(true);
                state.set_error_message.set(None);
                match crate::api::commands::import_templates(names_clone).await {
                    Ok(result) => {
                        state.set_success_message.set(Some(
                            t("template.import_skipped")
                                .replace("{}", &result.imported.to_string())
                                .replace("{}", &result.skipped.to_string()),
                        ));
                        set_selected_templates.set(Vec::new());
                        // 延迟清除成功消息
                        set_timeout(
                            move || {
                                state.set_success_message.set(None);
                            },
                            std::time::Duration::from_secs(3),
                        );
                    },
                    Err(e) => {
                        state.set_error_message.set(Some(e.display()));
                    },
                }
                state.set_loading.set(false);
            });
        }
    };

    // 自定义模板新建表单
    let (new_name, set_new_name) = signal(String::new());
    let (new_command, set_new_command) = signal(String::new());
    let (new_desc, set_new_desc) = signal(String::new());

    // 重新加载模板（按当前分类），供新建/删除后刷新。
    let reload_templates = move || {
        let category = active_category.get();
        let category_str = category.map(|c| c.to_string());
        let set_templates = set_templates;
        spawn_local(async move {
            match crate::api::commands::list_templates(category_str).await {
                Ok(t) => set_templates.set(t),
                Err(e) => log::warn!("Failed to reload templates: {}", e),
            }
        });
    };

    let on_save_custom = {
        move || {
            let name = new_name.get();
            let command = new_command.get();
            let desc = new_desc.get();
            if name.trim().is_empty() || command.trim().is_empty() {
                state
                    .set_error_message
                    .set(Some(t("template.custom_required").to_string()));
                return;
            }
            let template = crate::state::app_state::Template {
                name: name.trim().to_string(),
                command: command.trim().to_string(),
                description: desc.trim().to_string(),
                category: TemplateCategory::Custom,
                tags: vec![],
            };
            spawn_local(async move {
                match crate::api::commands::save_template(template).await {
                    Ok(()) => {
                        set_new_name.set(String::new());
                        set_new_command.set(String::new());
                        set_new_desc.set(String::new());
                        reload_templates();
                    },
                    Err(e) => state.set_error_message.set(Some(e.display())),
                }
            });
        }
    };

    let on_delete_custom = {
        move |name: String| {
            spawn_local(async move {
                match crate::api::commands::delete_template(name).await {
                    Ok(()) => reload_templates(),
                    Err(e) => state.set_error_message.set(Some(e.display())),
                }
            });
        }
    };

    view! {
        <div class="app-header">
            <h1 class="app-header__title">{move || t("template.title")}</h1>
            <div class="app-header__actions">
                <button
                    class="btn btn--primary"
                    on:click=move |_| on_import()
                    disabled=move || selected_templates.get().is_empty()
                >
                    {move || t("template.import_selected")}
                </button>
            </div>
        </div>

        <div class="app-content">
            {
                move || {
                    let success = state.success_message.get();
                    if let Some(msg) = success {
                        view! {
                            <div class="alert alert--success mb-lg">
                                {msg}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }
            }

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

            <TemplateCategoryTabs
                active_category=active_category
                on_select=Callback::new(move |cat: Option<TemplateCategory>| set_active_category.set(cat))
            />

            // 仅当选中 Custom 分类时展示「新建自定义模板」表单
            {
                move || {
                    if active_category.get() == Some(TemplateCategory::Custom) {
                        view! {
                            <div class="custom-template-form">
                                <div class="custom-template-form__title">{t("template.custom_new")}</div>
                                <div class="custom-template-form__fields">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder=t("template.custom_name")
                                        prop:value=move || new_name.get()
                                        on:input=move |e| set_new_name.set(event_target_value(&e))
                                    />
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder=t("template.custom_command")
                                        prop:value=move || new_command.get()
                                        on:input=move |e| set_new_command.set(event_target_value(&e))
                                    />
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder=t("template.custom_desc")
                                        prop:value=move || new_desc.get()
                                        on:input=move |e| set_new_desc.set(event_target_value(&e))
                                    />
                                    <button class="btn btn--primary" on:click=move |_| on_save_custom()>
                                        {t("template.custom_save")}
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }
            }

            <TemplateList
                templates=templates
                selected=selected_templates
                on_toggle=Callback::new(move |name: String| {
                    let mut current = selected_templates.get().to_vec();
                    if let Some(pos) = current.iter().position(|n| n == &name) {
                        current.remove(pos);
                    } else {
                        current.push(name);
                    }
                    set_selected_templates.set(current);
                })
                on_delete=Callback::new(on_delete_custom)
            />
        </div>
    }
}

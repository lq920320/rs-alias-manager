/// 别名表单组件（模态对话框）。
///
/// 提供添加或编辑别名的表单，包含验证：
/// - 名称：必填，仅允许字母数字+连字符+下划线
/// - 命令：必填
/// - 标签：可选，逗号分隔
use leptos::prelude::*;

/// 别名表单模态组件。
///
/// # 属性
/// * `alias` - 如果为 `Some`，则表单处于编辑模式并预填充值
/// * `on_submit` - 提交时的回调 (old_name, name, command, tags)
/// * `on_cancel` - 用户取消时的回调
#[component]
pub fn AliasForm(
    alias: Option<(String, String, Vec<String>)>,
    on_submit: Callback<(Option<String>, String, String, Vec<String>)>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let is_edit = alias.is_some();
    let (name, set_name) = signal(alias.as_ref().map(|a| a.0.clone()).unwrap_or_default());
    let (command, set_command) = signal(alias.as_ref().map(|a| a.1.clone()).unwrap_or_default());
    let (tags_str, set_tags_str) =
        signal(alias.as_ref().map(|a| a.2.join(", ")).unwrap_or_default());
    let (name_error, set_name_error) = signal(None::<String>);
    let (command_error, set_command_error) = signal(None::<String>);

    let validate_name = move |n: &str| -> Result<(), String> {
        if n.is_empty() {
            return Err("别名名称不能为空".to_string());
        }
        if n.starts_with('-') {
            return Err("别名名称不能以连字符开头".to_string());
        }
        if !n.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("别名名称只能包含字母、数字、下划线和连字符".to_string());
        }
        Ok(())
    };

    let handle_submit = move || {
        let n = name.get();
        let c = command.get();
        let t = tags_str.get();

        let name_valid = match validate_name(&n) {
            Ok(()) => {
                set_name_error.set(None);
                true
            },
            Err(e) => {
                set_name_error.set(Some(e));
                false
            },
        };

        let command_valid = if c.is_empty() {
            set_command_error.set(Some("命令不能为空".to_string()));
            false
        } else {
            set_command_error.set(None);
            true
        };

        if name_valid && command_valid {
            let tags: Vec<String> =
                t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            let old_name = alias.as_ref().map(|a| a.0.clone());
            on_submit.run((old_name, n, c, tags));
        }
    };

    let title = if is_edit { "编辑别名" } else { "添加别名" };

    view! {
        <div class="modal-overlay" on:click=move |_| on_cancel.run(())>
            <div class="modal" on:click=|e| e.stop_propagation()>
                <div class="modal__header">
                    <h2 class="modal__title">{ title }</h2>
                    <button class="modal__close" on:click=move |_| on_cancel.run(())>
                        "✕"
                    </button>
                </div>
                <div class="modal__body">
                    <div class="form-group">
                        <label class="form-group__label form-group__label--required">
                            "别名名称"
                        </label>
                        <input
                            class=format!("form-group__input{}", if name_error.get().is_some() { " form-group__input--error" } else { "" })
                            type="text"
                            placeholder="例如: gs"
                            prop:value=move || name.get()
                            on:input=move |e| set_name.set(event_target_value(&e))
                            disabled=is_edit
                        />
                        {
                            move || {
                                name_error.get().map(|e| view! {
                                    <div class="form-group__error">{ e }</div>
                                })
                            }
                        }
                        <div class="form-group__hint">
                            "只能包含字母、数字、下划线和连字符"
                        </div>
                    </div>

                    <div class="form-group">
                        <label class="form-group__label form-group__label--required">
                            "命令"
                        </label>
                        <textarea
                            class=format!("form-group__textarea{}", if command_error.get().is_some() { " form-group__input--error" } else { "" })
                            placeholder="例如: git status"
                            prop:value=move || command.get()
                            on:input=move |e| set_command.set(event_target_value(&e))
                        ></textarea>
                        {
                            move || {
                                command_error.get().map(|e| view! {
                                    <div class="form-group__error">{ e }</div>
                                })
                            }
                        }
                    </div>

                    <div class="form-group">
                        <label class="form-group__label">"标签"</label>
                        <input
                            class="form-group__input"
                            type="text"
                            placeholder="例如: git, 快捷命令（逗号分隔）"
                            prop:value=move || tags_str.get()
                            on:input=move |e| set_tags_str.set(event_target_value(&e))
                        />
                        <div class="form-group__hint">
                            "用逗号分隔多个标签"
                        </div>
                    </div>
                </div>
                <div class="modal__footer">
                    <button class="btn btn--secondary" on:click=move |_| on_cancel.run(())>
                        "取消"
                    </button>
                    <button class="btn btn--primary" on:click=move |_| handle_submit()>
                        { if is_edit { "保存" } else { "添加" } }
                    </button>
                </div>
            </div>
        </div>
    }
}

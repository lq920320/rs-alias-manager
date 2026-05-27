/// 设置表单组件。
///
/// 提供配置应用程序设置的表单：
/// - Shell 类型选择
/// - 自定义配置文件路径
/// - 自动刷新开关
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::app_state::{AppState, ShellType};

fn set_timeout(f: impl FnOnce() + 'static, dur: std::time::Duration) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    let window = web_sys::window().unwrap();
    let cb = Closure::once_into_js(move || f());
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.unchecked_ref(),
        dur.as_millis() as i32,
    );
}

/// 设置表单组件。
#[component]
pub fn SettingsForm() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");

    let (custom_path, set_custom_path) = signal(String::new());
    let (save_message, set_save_message) = signal(None::<String>);

    // 从设置中初始化自定义路径
    Effect::new(move || {
        let settings = state.settings.get();
        set_custom_path.set(settings.custom_config_path.clone().unwrap_or_default());
    });

    let save_shell_type = move |shell_str: String| {
        let state = state;
        spawn_local(async move {
            match crate::api::commands::update_settings(Some(shell_str), None, None).await {
                Ok(settings) => {
                    let shell_type = settings.shell_type;
                    state.set_settings.set(settings);
                    state.set_shell_type.set(shell_type);
                    // 更新配置路径
                    match crate::api::commands::get_config_file_path().await {
                        Ok(path) => state.set_config_path.set(path),
                        Err(e) => log::warn!("Failed to get config path: {}", e),
                    }
                    set_save_message.set(Some("Shell 类型已更新".to_string()));
                    set_timeout(
                        move || set_save_message.set(None),
                        std::time::Duration::from_secs(3),
                    );
                },
                Err(e) => {
                    state.set_error_message.set(Some(e));
                },
            }
        });
    };

    let save_custom_path = move || {
        let state = state;
        let path = custom_path.get();
        let path_opt = if path.is_empty() { None } else { Some(path) };
        spawn_local(async move {
            match crate::api::commands::update_settings(None, path_opt, None).await {
                Ok(settings) => {
                    state.set_settings.set(settings);
                    match crate::api::commands::get_config_file_path().await {
                        Ok(p) => state.set_config_path.set(p),
                        Err(e) => log::warn!("Failed to get config path: {}", e),
                    }
                    set_save_message.set(Some("配置路径已更新".to_string()));
                    set_timeout(
                        move || set_save_message.set(None),
                        std::time::Duration::from_secs(3),
                    );
                },
                Err(e) => {
                    state.set_error_message.set(Some(e));
                },
            }
        });
    };

    let save_auto_refresh = move |value: bool| {
        let state = state;
        spawn_local(async move {
            match crate::api::commands::update_settings(None, None, Some(value)).await {
                Ok(settings) => {
                    state.set_settings.set(settings);
                },
                Err(e) => {
                    state.set_error_message.set(Some(e));
                },
            }
        });
    };

    let reload_aliases = move || {
        let state = state;
        spawn_local(async move {
            state.set_loading.set(true);
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
    };

    view! {
        <div class="settings-form">
            {
                move || {
                    save_message.get().map(|msg| view! {
                        <div class="alert alert--success mb-lg">{ msg }</div>
                    })
                }
            }

            <div class="settings-form__section">
                <h3 class="settings-form__section-title">"Shell 配置"</h3>

                <div class="settings-form__row">
                    <div>
                        <div class="settings-form__label">"Shell 类型"</div>
                        <div class="settings-form__description">
                            "选择你要管理的 Shell 配置文件"
                        </div>
                    </div>
                    <select
                        class="form-group__select"
                        style="width:160px"
                        on:change=move |e| {
                            let val = event_target_value(&e);
                            save_shell_type(val);
                        }
                    >
                        {
                            move || {
                                let current = state.shell_type.get();
                                ShellType::all().into_iter().map(|st| {
                                    let value = st.to_string();
                                    let selected = st == current;
                                    view! {
                                        <option value=value.clone() selected=selected>
                                            { st.label() }
                                        </option>
                                    }
                                }).collect::<Vec<_>>()
                            }
                        }
                    </select>
                </div>

                <div class="settings-form__row">
                    <div style="flex:1;margin-right:16px">
                        <div class="settings-form__label">"自定义配置路径"</div>
                        <div class="settings-form__description">
                            "留空则使用默认路径"
                        </div>
                    </div>
                    <div style="display:flex;gap:8px;align-items:center">
                        <input
                            class="form-group__input"
                            type="text"
                            style="width:300px"
                            placeholder="例如: /home/user/.custom_bashrc"
                            prop:value=move || custom_path.get()
                            on:input=move |e| set_custom_path.set(event_target_value(&e))
                        />
                        <button class="btn btn--primary btn--sm" on:click=move |_| save_custom_path()>
                            "保存"
                        </button>
                    </div>
                </div>
            </div>

            <div class="settings-form__section">
                <h3 class="settings-form__section-title">"数据管理"</h3>

                <div class="settings-form__row">
                    <div>
                        <div class="settings-form__label">"自动刷新"</div>
                        <div class="settings-form__description">
                            "配置文件变更时自动刷新别名列表"
                        </div>
                    </div>
                    <label class="toggle">
                        <input
                            type="checkbox"
                            class="toggle__input"
                            checked=move || state.settings.get().auto_refresh
                            on:change=move |e| {
                                let checked = event_target_checked(&e);
                                save_auto_refresh(checked);
                            }
                        />
                        <span class="toggle__slider"></span>
                    </label>
                </div>

                <div class="settings-form__row">
                    <div>
                        <div class="settings-form__label">"手动刷新"</div>
                        <div class="settings-form__description">
                            "立即重新读取配置文件"
                        </div>
                    </div>
                    <button class="btn btn--secondary btn--sm" on:click=move |_| reload_aliases()>
                        "刷新"
                    </button>
                </div>
            </div>

            <div class="settings-form__section">
                <h3 class="settings-form__section-title">"关于"</h3>
                <div class="card">
                    <div style="color:var(--text-secondary);font-size:14px">
                        <div style="margin-bottom:8px">
                            <strong>"rs-alias-manager"</strong>
                            " v0.1.0"
                        </div>
                        <div>"基于 Tauri v2 + Leptos 0.8 构建的 Shell 别名管理器"</div>
                        <div style="margin-top:8px">
                            "支持 Bash、Zsh、Fish 配置文件管理"
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

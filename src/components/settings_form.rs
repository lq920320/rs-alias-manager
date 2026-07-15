/// 设置表单组件。
///
/// 提供配置应用程序设置的表单：
/// - Shell 类型选择
/// - 自定义配置文件路径
/// - 自动刷新开关
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::i18n::{t, Locale};
use crate::state::app_state::{AppState, ShellType};
use crate::utils::set_timeout;

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
            match crate::api::commands::update_settings(Some(shell_str), None, None, None).await {
                Ok(settings) => {
                    let shell_type = settings.shell_type;
                    state.set_settings.set(settings);
                    state.set_shell_type.set(shell_type);
                    // 更新配置路径
                    match crate::api::commands::get_config_file_path().await {
                        Ok(path) => state.set_config_path.set(path),
                        Err(e) => log::warn!("Failed to get config path: {}", e),
                    }
                    set_save_message.set(Some(t("settings.shell_updated")));
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
            match crate::api::commands::update_settings(None, path_opt, None, None).await {
                Ok(settings) => {
                    state.set_settings.set(settings);
                    match crate::api::commands::get_config_file_path().await {
                        Ok(p) => state.set_config_path.set(p),
                        Err(e) => log::warn!("Failed to get config path: {}", e),
                    }
                    set_save_message.set(Some(t("settings.path_updated")));
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
            match crate::api::commands::update_settings(None, None, Some(value), None).await {
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
                <h3 class="settings-form__section-title">{move || t("settings.language")}</h3>

                <div class="settings-form__row">
                    <div>
                        <div class="settings-form__label">{move || t("settings.language")}</div>
                        <div class="settings-form__description">
                            {move || t("settings.language_desc")}
                        </div>
                    </div>
                    <select
                        class="form-group__select"
                        style="width:160px"
                        on:change=move |e| {
                            let val = event_target_value(&e);
                            let state = state;
                            spawn_local(async move {
                                match crate::api::commands::update_settings(None, None, None, Some(val.clone())).await {
                                    Ok(settings) => {
                                        if let Ok(loc) = val.parse::<Locale>() {
                                            state.set_locale.set(loc);
                                        }
                                        state.set_settings.set(settings);
                                        set_save_message.set(Some(t("settings.language_updated")));
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
                        }
                    >
                        {
                            move || {
                                let current = state.locale.get();
                                Locale::all().into_iter().map(|loc| {
                                    let value = loc.to_string();
                                    let selected = loc == current;
                                    let label = loc.label();
                                    view! {
                                        <option value=value selected=selected>
                                            { label }
                                        </option>
                                    }
                                }).collect::<Vec<_>>()
                            }
                        }
                    </select>
                </div>
            </div>

            <div class="settings-form__section">
                <h3 class="settings-form__section-title">{move || t("settings.shell_config")}</h3>

                <div class="settings-form__row">
                    <div>
                        <div class="settings-form__label">{move || t("settings.shell_type")}</div>
                        <div class="settings-form__description">
                            {move || t("settings.shell_type_desc")}
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
                        <div class="settings-form__label">{move || t("settings.custom_path")}</div>
                        <div class="settings-form__description">
                            {move || t("settings.custom_path_desc")}
                        </div>
                    </div>
                    <div style="display:flex;gap:8px;align-items:center">
                        <input
                            class="form-group__input"
                            type="text"
                            style="width:300px"
                            placeholder=move || t("settings.custom_path_placeholder")
                            prop:value=move || custom_path.get()
                            on:input=move |e| set_custom_path.set(event_target_value(&e))
                        />
                        <button class="btn btn--primary btn--sm" on:click=move |_| save_custom_path()>
                            {move || t("settings.save")}
                        </button>
                    </div>
                </div>
            </div>

            <div class="settings-form__section">
                <h3 class="settings-form__section-title">{move || t("settings.data_management")}</h3>

                <div class="settings-form__row">
                    <div>
                        <div class="settings-form__label">{move || t("settings.auto_refresh")}</div>
                        <div class="settings-form__description">
                            {move || t("settings.auto_refresh_desc")}
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
                        <div class="settings-form__label">{move || t("settings.manual_refresh")}</div>
                        <div class="settings-form__description">
                            {move || t("settings.manual_refresh_desc")}
                        </div>
                    </div>
                    <button class="btn btn--secondary btn--sm" on:click=move |_| reload_aliases()>
                        {move || t("settings.refresh_btn")}
                    </button>
                </div>
            </div>

            <div class="settings-form__section">
                <h3 class="settings-form__section-title">{move || t("settings.about")}</h3>
                <div class="card">
                    <div style="color:var(--text-secondary);font-size:14px">
                        <div style="margin-bottom:8px">
                            <strong>"rs-alias-manager"</strong>
                            " v0.1.0"
                        </div>
                        <div>{move || t("settings.about_desc")}</div>
                        <div style="margin-top:8px">
                            {move || t("settings.about_support")}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

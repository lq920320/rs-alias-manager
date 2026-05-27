/// 设置页面。
///
/// 允许用户配置应用程序：
/// - 选择 Shell 类型（Bash/Zsh/Fish）
/// - 设置自定义配置文件路径
/// - 切换自动刷新
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::settings_form::SettingsForm;
use crate::state::app_state::AppState;

/// 设置页面组件。
#[component]
pub fn SettingsPage() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should been provided");

    // 挂载时加载设置
    Effect::new(move || {
        let state = state;
        spawn_local(async move {
            match crate::api::commands::get_settings().await {
                Ok(settings) => {
                    state.set_settings.set(settings);
                },
                Err(e) => {
                    state.set_error_message.set(Some(e));
                },
            }
            match crate::api::commands::get_config_file_path().await {
                Ok(path) => {
                    state.set_config_path.set(path);
                },
                Err(e) => {
                    log::warn!("Failed to get config path: {}", e);
                },
            }
        });
    });

    view! {
        <div class="app-header">
            <h1 class="app-header__title">"设置"</h1>
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

            <SettingsForm />
        </div>
    }
}

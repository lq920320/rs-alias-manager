/// Tauri 后端的库入口点。
///
/// 注册所有命令和插件，然后启动应用程序。
mod commands;
pub mod error;
mod models;
pub mod services;
pub mod state;

use state::AppState;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::Manager;

/// 初始化并运行 Tauri 应用程序。
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Resolve the app data directory at startup and store it as managed state
            let app_data_dir = app.path().app_data_dir()?;
            app.manage(AppState::new(app_data_dir));

            // 构建应用菜单（左上角），在「关于」下方加入「检查更新」项
            let check_updates = MenuItemBuilder::with_id("check_updates", "检查更新...")
                .build(app)?;

            #[cfg(target_os = "macos")]
            let app_submenu = SubmenuBuilder::new(app, "rs-alias-manager")
                .about(None)
                .item(&check_updates)
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()?;

            #[cfg(not(target_os = "macos"))]
            let app_submenu = SubmenuBuilder::new(app, "rs-alias-manager")
                .about(None)
                .item(&check_updates)
                .separator()
                .quit()
                .build()?;

            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let window_submenu = SubmenuBuilder::new(app, "Window")
                .minimize()
                .separator()
                .close_window()
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&edit_submenu)
                .item(&window_submenu)
                .build()?;

            app.set_menu(menu)?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == "check_updates" {
                handle_check_updates_menu(app);
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Alias commands
            commands::alias_cmds::list_aliases,
            commands::alias_cmds::add_alias,
            commands::alias_cmds::update_alias,
            commands::alias_cmds::delete_alias,
            commands::alias_cmds::detect_shell,
            // Batch commands
            commands::alias_cmds::batch_add_aliases,
            commands::alias_cmds::batch_delete_aliases,
            // Template commands
            commands::template_cmds::list_templates,
            commands::template_cmds::import_templates,
            // Settings commands
            commands::settings_cmds::get_settings,
            commands::settings_cmds::update_settings,
            commands::settings_cmds::get_config_file_path,
            // Update commands
            commands::update_cmds::check_for_updates,
            commands::update_cmds::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 处理「检查更新」菜单事件。
///
/// 在后台线程中执行网络请求（ureq 为阻塞调用），完成后根据结果弹出原生对话框：
/// - 版本一致：「当前已是最新版本」
/// - 发现新版本：询问是否前往下载，确认则用系统浏览器打开发布页
/// - 请求失败：显示错误信息
fn handle_check_updates_menu(app: &tauri::AppHandle) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
    use tauri_plugin_shell::ShellExt;

    // 读取当前界面语言，决定对话框文案
    let is_zh = app
        .try_state::<AppState>()
        .map(|s| s.get_settings().locale == "zh")
        .unwrap_or(false);

    let app_handle = app.clone();
    std::thread::spawn(move || {
        // 从 tauri.conf.json 读取当前版本（打包时的权威版本）
        let current = app_handle.package_info().version.to_string();
        let title = if is_zh { "检查更新" } else { "Check for Updates" };

        match crate::services::update_checker::check_for_updates(&current) {
            Ok(info) => {
                if info.has_update {
                    let url = info.release_url.clone();
                    let msg = if is_zh {
                        format!("发现新版本！\n\n最新版本: v{}\n\n是否前往下载？", info.latest_version)
                    } else {
                        format!(
                            "A new version is available!\n\nLatest version: v{}\n\nDownload now?",
                            info.latest_version
                        )
                    };
                    let app_for_cb = app_handle.clone();
                    app_handle
                        .dialog()
                        .message(msg)
                        .title(title)
                        .buttons(MessageDialogButtons::YesNo)
                        .show(move |confirmed| {
                            if confirmed {
                                // shell().open() 在新版中标记为 deprecated（建议改用 opener 插件），
                                // 但本应用已集成 shell 插件，这里继续使用以避免引入额外依赖。
                                #[allow(deprecated)]
                                let _ = app_for_cb.shell().open(url, None);
                            }
                        });
                } else {
                    let msg = if is_zh {
                        format!("当前已是最新版本\n\n最新版本: v{}", info.latest_version)
                    } else {
                        format!("You're up to date\n\nLatest version: v{}", info.latest_version)
                    };
                    app_handle.dialog().message(msg).title(title).show(|_| {});
                }
            },
            Err(e) => {
                let msg = if is_zh {
                    format!("检查更新失败\n\n{}", e)
                } else {
                    format!("Failed to check for updates\n\n{}", e)
                };
                app_handle.dialog().message(msg).title(title).show(|_| {});
            },
        }
    });
}

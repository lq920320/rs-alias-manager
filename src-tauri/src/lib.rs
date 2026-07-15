/// Tauri 后端的库入口点。
/// 注册所有命令和插件，然后启动应用程序。
mod commands;
pub mod error;
mod models;
pub mod services;
pub mod state;

use state::AppState;
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
            Ok(())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

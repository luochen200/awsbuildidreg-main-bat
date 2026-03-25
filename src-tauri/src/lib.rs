mod browser_automation;
mod builder_id_automation;
mod commands;
mod database;
mod graph_api;
mod imap_client;
mod integrated_registration;
mod kiro_auth;
mod kiro_oauth;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = database::init_database(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_accounts,
            commands::add_account,
            commands::update_account,
            commands::delete_account,
            commands::delete_all_accounts,
            commands::import_accounts,
            commands::get_settings,
            commands::update_settings,
            commands::start_registration,
            commands::start_batch_registration,
            commands::export_accounts,
            commands::generate_kiro_social_auth,
            commands::generate_kiro_idc_auth,
            commands::read_kiro_auth_token,
            commands::simulate_kiro_login,
            commands::kiro_oauth_login_google,
            commands::kiro_oauth_login_github,
            commands::kiro_oauth_login_builder_id,
            commands::handle_oauth_callback_url,
            commands::builder_id_automated_login,
            commands::start_registration_with_oauth,
            commands::get_oauth_info,
            commands::manual_oauth_authorization,
            commands::export_kiro_auth_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

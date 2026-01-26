#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod file_tracker;
mod drive_sync;
mod version_manager;
mod scheduler;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize configuration
            if let Err(e) = config::init_config() {
                eprintln!("Failed to initialize config: {}", e);
            }

            // Initialize file tracker database
            if let Err(e) = file_tracker::init_database() {
                eprintln!("Failed to initialize database: {}", e);
            }

            // Start scheduler if enabled
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                scheduler::start_scheduler(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_staging_dir,
            commands::set_drive_folder,
            commands::set_sync_interval,
            commands::set_auto_sync,
            commands::get_tracked_paths,
            commands::add_tracked_path,
            commands::remove_tracked_path,
            commands::sync_now,
            commands::get_sync_status,
            commands::get_auth_url,
            commands::open_url,
            commands::handle_oauth_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod file_tracker;
mod drive_sync;
mod version_manager;
mod scheduler;
mod logger;

use tauri::{Manager, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}};
use tauri::Emitter;

fn main() {
    // Initialize logger first
    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }
    
    logger::log_info("Starting Sync Bot application...");
    logger::log_info("Initializing Tauri plugins...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            logger::log_info("Tauri setup started");
            
            // Initialize configuration
            logger::log_info("Initializing configuration...");
            if let Err(e) = config::init_config() {
                let msg = format!("Failed to initialize config: {}", e);
                eprintln!("{}", msg);
                logger::log_error(&msg);
            } else {
                logger::log_info("Configuration initialized successfully");
            }

            // Initialize file tracker database
            logger::log_info("Initializing file tracker database...");
            if let Err(e) = file_tracker::init_database() {
                let msg = format!("Failed to initialize database: {}", e);
                eprintln!("{}", msg);
                logger::log_error(&msg);
            } else {
                logger::log_info("File tracker database initialized successfully");
            }

            // Start scheduler if enabled (using Tauri's async runtime)
            logger::log_info("Starting scheduler...");
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::start_scheduler(app_handle).await;
            });

            // Setup system tray
            logger::log_info("Setting up system tray...");
            setup_tray(app)?;

            // Handle window close event to minimize to tray
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent window from closing, hide it instead
                    api.prevent_close();
                    let _ = window_clone.hide();
                    logger::log_info("Window hidden to system tray");
                }
            });

            logger::log_info("Tauri setup completed successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_version,
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
                    commands::check_auth_status,
                    commands::listen_for_oauth_code,
                    commands::set_google_client_id,
            commands::set_google_client_secret,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    logger::log_info("=== Sync Bot Stopped ===");
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create tray menu
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let sync_item = MenuItem::with_id(app, "sync", "Sync Now", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[&show_item, &sync_item, &quit_item])?;
    
    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        logger::log_info("Window shown from tray");
                    }
                }
                "sync" => {
                    logger::log_info("Sync triggered from tray menu");
                    let _ = app.emit("tray-sync-requested", ());
                }
                "quit" => {
                    logger::log_info("Quit requested from tray");
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;
    
    Ok(())
}

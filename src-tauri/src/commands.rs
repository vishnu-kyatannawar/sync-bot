use serde::Serialize;
use std::path::PathBuf;
use anyhow::Result;
use zip::{write::FileOptions, CompressionMethod};

#[tauri::command]
pub fn get_config() -> Result<crate::config::Config, String> {
    crate::logger::log_info("Command: get_config called");
    let result = crate::config::load_config()
        .map_err(|e| {
            let msg = format!("get_config error: {}", e);
            crate::logger::log_error(&msg);
            e.to_string()
        });
    if result.is_ok() {
        crate::logger::log_info("get_config completed successfully");
    }
    result
}

#[tauri::command]
pub fn set_staging_dir(path: String) -> Result<(), String> {
    crate::logger::log_info(&format!("Command: set_staging_dir called with path: {}", path));
    crate::config::update_config(|config| {
        config.staging_dir = Some(path.clone());
    })
    .map_err(|e| {
        let msg = format!("set_staging_dir error: {}", e);
        crate::logger::log_error(&msg);
        e.to_string()
    })?;
    crate::logger::log_info("set_staging_dir completed successfully");
    Ok(())
}

#[tauri::command]
pub fn set_drive_folder(folder: String) -> Result<(), String> {
    crate::config::update_config(|config| {
        config.drive_folder = Some(folder);
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_sync_interval(interval: u64) -> Result<(), String> {
    crate::config::update_config(|config| {
        config.sync_interval = Some(interval);
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_auto_sync(enabled: bool) -> Result<(), String> {
    crate::config::update_config(|config| {
        config.auto_sync = Some(enabled);
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_tracked_paths() -> Result<Vec<String>, String> {
    crate::file_tracker::get_tracked_paths()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tracked_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    
    crate::file_tracker::add_tracked_path(&path_buf)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_tracked_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    crate::file_tracker::remove_tracked_path(&path_buf)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct SyncResult {
    files_synced: usize,
    files_skipped: usize,
    errors: Vec<String>,
}

#[tauri::command]
pub async fn sync_now() -> Result<SyncResult, String> {
    crate::logger::log_info("=== Sync Now Command Started ===");
    
    // Get configuration
    let config = crate::config::load_config()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    
    let drive_folder = config.drive_folder
        .unwrap_or_else(|| "sync-bot-backups".to_string());
    
    // Get staging directory
    let staging_dir = crate::config::get_staging_dir()
        .map_err(|e| format!("Failed to get staging directory: {}", e))?;
    
    crate::logger::log_info(&format!("Staging directory: {:?}", staging_dir));
    
    // First, ensure all tracked files are copied to staging
    let tracked_paths = crate::file_tracker::get_tracked_paths()
        .map_err(|e| format!("Failed to get tracked paths: {}", e))?;
    
    if tracked_paths.is_empty() {
        crate::logger::log_warn("No files or folders are being tracked");
        return Ok(SyncResult {
            files_synced: 0,
            files_skipped: 0,
            errors: vec!["No files or folders tracked. Please add files/folders first.".to_string()],
        });
    }
    
    crate::logger::log_info(&format!("Found {} tracked path(s)", tracked_paths.len()));
    
    // Copy all tracked files to staging if needed
    let files = crate::file_tracker::get_all_files_to_sync()
        .map_err(|e| format!("Failed to get files to sync: {}", e))?;
    
    crate::logger::log_info(&format!("Total files to process: {}", files.len()));
    
    for file_path in &files {
        if !file_path.starts_with(&staging_dir) {
            // File is outside staging, need to copy it
            let tracked_paths = crate::file_tracker::get_tracked_paths()
                .map_err(|e| format!("Failed to get tracked paths: {}", e))?;
            
            let mut found_base = None;
            for tracked in &tracked_paths {
                let tracked_buf = PathBuf::from(tracked);
                if file_path.starts_with(&tracked_buf) {
                    found_base = Some(tracked_buf);
                    break;
                }
            }
            
            let relative_path = if let Some(ref base) = found_base {
                if let Ok(rel) = file_path.strip_prefix(base) {
                    if rel.as_os_str().is_empty() {
                        // Base is the file itself (single file tracked)
                        let file_name = file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        staging_dir.join("tracked").join(file_name)
                    } else {
                        // Base is a directory, preserve its structure
                        // Get the base folder name to maintain structure in ZIP
                        if let Some(base_name) = base.file_name() {
                            // Include base folder name: tracked/base_name/relative_path
                            staging_dir.join("tracked").join(base_name).join(rel)
                        } else {
                            // Fallback: just use relative path
                            staging_dir.join("tracked").join(rel)
                        }
                    }
                } else {
                    // Fallback: use filename
                    let file_name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    staging_dir.join("tracked").join(file_name)
                }
            } else {
                // No base found, use filename
                let file_name = file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                staging_dir.join("tracked").join(file_name)
            };
            
            if let Some(parent) = relative_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create staging directory: {}", e))?;
            }
            
            // Copy file to staging
            if relative_path.exists() {
                // If file exists, try to make it writable first to avoid permission denied errors
                // when overwriting read-only files (like SSH keys)
                if let Ok(metadata) = std::fs::metadata(&relative_path) {
                    let mut permissions = metadata.permissions();
                    if permissions.readonly() {
                        #[allow(clippy::permissions_set_readonly_false)]
                        permissions.set_readonly(false);
                        let _ = std::fs::set_permissions(&relative_path, permissions);
                    }
                }
                // Alternatively, remove the file first to ensure we can copy fresh
                let _ = std::fs::remove_file(&relative_path);
            }

            if let Err(e) = std::fs::copy(file_path, &relative_path) {
                let msg = format!("Failed to copy {} to {}: {}", file_path.display(), relative_path.display(), e);
                crate::logger::log_error(&msg);
                return Err(msg);
            }
            
            crate::logger::log_info(&format!("Copied {} to staging", file_path.display()));
        }
    }
    
    // Create archive before sync (for version history)
    let archives_dir = crate::config::get_archives_dir()
        .map_err(|e| format!("Failed to get archives directory: {}", e))?;
    
    if let Err(e) = crate::version_manager::create_archive(&staging_dir, &archives_dir) {
        crate::logger::log_warn(&format!("Warning: Failed to create archive: {}", e));
    }
    
    // Create ZIP of staging directory for sync
    crate::logger::log_info("Creating ZIP file of staging directory...");
    let zip_path = staging_dir.join("backup.zip");
    
    // Check if ZIP exists and has changed
    let zip_needs_sync = if zip_path.exists() {
        // Check if staging directory has changed since last ZIP creation
        let zip_modified = std::fs::metadata(&zip_path)
            .and_then(|m| m.modified())
            .ok();
        
        let staging_modified = std::fs::read_dir(&staging_dir)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .filter_map(|m| m.modified().ok())
                    .max()
            });
        
        match (zip_modified, staging_modified) {
            (Some(zip_time), Some(staging_time)) => staging_time > zip_time,
            _ => true, // If we can't determine, create new ZIP
        }
    } else {
        true // ZIP doesn't exist, need to create it
    };
    
    if zip_needs_sync {
        crate::logger::log_info("Staging directory has changed, creating new ZIP...");
        
        // Create ZIP file
        let file = std::fs::File::create(&zip_path)
            .map_err(|e| format!("Failed to create ZIP file: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        
        // Add files from the "tracked" subdirectory, but without the "tracked" folder wrapper
        let tracked_dir = staging_dir.join("tracked");
        if tracked_dir.exists() {
            crate::version_manager::add_directory_to_zip(&mut zip, &tracked_dir, &tracked_dir, "")
                .map_err(|e| format!("Failed to add files to ZIP: {}", e))?;
        }
        
        zip.finish()
            .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;
        
        crate::logger::log_info(&format!("ZIP file created: {:?}", zip_path));
    } else {
        crate::logger::log_info("Staging directory unchanged, using existing ZIP");
    }
    
    // Check if ZIP has changed (for smart sync)
    let zip_changed = crate::file_tracker::has_file_changed(&zip_path)
        .map_err(|e| format!("Failed to check if ZIP changed: {}", e))?;
    
    if !zip_changed {
        crate::logger::log_info("ZIP file has not changed, skipping upload");
        
        // Still update last sync time to show we checked
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = crate::file_tracker::set_metadata("last_sync_time", &now.to_string());

        return Ok(SyncResult {
            files_synced: 0,
            files_skipped: 1,
            errors: vec![],
        });
    }
    
    // Initialize Drive sync
    let mut drive_sync = crate::drive_sync::DriveSync::new();
    
    // Find or create folder in Google Drive
    let folder_id = drive_sync.find_or_create_folder(&drive_folder)
        .await
        .map_err(|e| format!("Failed to find/create Drive folder: {}", e))?;
    
    crate::logger::log_info(&format!("Drive folder ID: {}", folder_id));
    
    // Upload ZIP file to Drive
    crate::logger::log_info("Uploading ZIP file to Google Drive...");
    match drive_sync.upload_file(&zip_path, &folder_id).await {
        Ok(_) => {
            // Mark ZIP as synced
            if let Err(e) = crate::file_tracker::mark_file_synced(&zip_path) {
                crate::logger::log_error(&format!("Failed to mark ZIP as synced: {}", e));
            }
            
            // Save last sync time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = crate::file_tracker::set_metadata("last_sync_time", &now.to_string());

            crate::logger::log_info("ZIP file uploaded successfully!");
            Ok(SyncResult {
                files_synced: 1,
                files_skipped: 0,
                errors: vec![],
            })
        }
        Err(e) => {
            crate::logger::log_error(&format!("Failed to upload ZIP: {}", e));
            Ok(SyncResult {
                files_synced: 0,
                files_skipped: 0,
                errors: vec![format!("Failed to upload ZIP file: {}", e)],
            })
        }
    }
}

#[derive(Serialize)]
pub struct SyncStatus {
    last_sync: Option<u64>,
    next_sync: Option<u64>,
    is_syncing: bool,
}

#[tauri::command]
pub fn get_sync_status() -> Result<SyncStatus, String> {
    let last_sync = crate::file_tracker::get_metadata("last_sync_time")
        .unwrap_or(None)
        .and_then(|s| s.parse::<u64>().ok());
    
    // Calculate next sync if auto-sync is enabled
    let mut next_sync = None;
    if let Ok(config) = crate::config::load_config() {
        if config.auto_sync.unwrap_or(false) {
            if let Some(last) = last_sync {
                let interval_secs = config.sync_interval.unwrap_or(60) * 60;
                next_sync = Some(last + interval_secs);
            }
        }
    }

    Ok(SyncStatus {
        last_sync,
        next_sync,
        is_syncing: false,
    })
}

#[tauri::command]
pub fn get_auth_url() -> Result<String, String> {
    crate::drive_sync::DriveSync::get_auth_url()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    // Use system command to open URL (shell plugin doesn't have direct open method)
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn handle_oauth_code(code: String) -> Result<(), String> {
    let mut drive_sync = crate::drive_sync::DriveSync::new();
    drive_sync.handle_oauth_callback(&code)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn check_auth_status() -> bool {
    crate::drive_sync::DriveSync::is_authenticated()
}

#[tauri::command]
pub async fn listen_for_oauth_code() -> Result<String, String> {
    use tiny_http::{Server, Response};
    use url::Url;

    crate::logger::log_info("Starting local server to listen for OAuth code...");
    
    let server = Server::http(format!("127.0.0.1:{}", crate::drive_sync::REDIRECT_PORT))
        .map_err(|e| format!("Failed to start local server: {}", e))?;

    // Set a timeout for the server so it doesn't run forever if user cancels
    // We'll wait for one request
    if let Ok(request) = server.recv() {
        let url = format!("http://localhost{}", request.url());
        let parsed_url = Url::parse(&url).map_err(|e| format!("Failed to parse callback URL: {}", e))?;
        
        let code = parsed_url.query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.into_owned());

        if let Some(code) = code {
            let response = Response::from_string("Authentication successful! You can close this window and return to the app.");
            let _ = request.respond(response);
            crate::logger::log_info("OAuth code received successfully");
            return Ok(code);
        } else {
            let response = Response::from_string("Authentication failed! No code found in request.");
            let _ = request.respond(response);
            return Err("No code found in the callback URL".to_string());
        }
    }

    Err("Server closed without receiving a request".to_string())
}

#[tauri::command]
pub fn set_google_client_id(id: String) -> Result<(), String> {
    crate::config::update_config(|config| {
        config.client_id = Some(id);
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_google_client_secret(secret: String) -> Result<(), String> {
    crate::config::update_config(|config| {
        config.client_secret = Some(secret);
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

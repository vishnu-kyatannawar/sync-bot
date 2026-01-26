use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[tauri::command]
pub fn get_config() -> Result<crate::config::Config, String> {
    crate::config::load_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_staging_dir(path: String) -> Result<(), String> {
    crate::config::update_config(|config| {
        config.staging_dir = Some(path);
    })
    .map_err(|e| e.to_string())?;
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
    // Get configuration
    let config = crate::config::load_config()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    
    let drive_folder = config.drive_folder
        .unwrap_or_else(|| "sync-bot-backups".to_string());
    
    // Get staging directory
    let staging_dir = crate::config::get_staging_dir()
        .map_err(|e| format!("Failed to get staging directory: {}", e))?;
    
    // Create archive before sync
    let archives_dir = crate::config::get_archives_dir()
        .map_err(|e| format!("Failed to get archives directory: {}", e))?;
    
    if let Err(e) = crate::version_manager::create_archive(&staging_dir, &archives_dir) {
        eprintln!("Warning: Failed to create archive: {}", e);
    }
    
    // Get all files to sync
    let files = crate::file_tracker::get_all_files_to_sync()
        .map_err(|e| format!("Failed to get files to sync: {}", e))?;
    
    // Initialize Drive sync
    let mut drive_sync = crate::drive_sync::DriveSync::new();
    
    // Find or create folder in Google Drive
    let folder_id = drive_sync.find_or_create_folder(&drive_folder)
        .await
        .map_err(|e| format!("Failed to find/create Drive folder: {}", e))?;
    
    let mut files_synced = 0;
    let mut files_skipped = 0;
    let mut errors = Vec::new();
    
    // Process each file
    for file_path in files {
        // Check if file has changed
        match crate::file_tracker::has_file_changed(&file_path) {
            Ok(true) => {
                // File has changed, need to sync
                // Determine relative path for staging
                let staging_file = if file_path.starts_with(&staging_dir) {
                    // File is already in staging
                    file_path.clone()
                } else {
                    // File is outside staging, need to copy it
                    // Get relative path from original tracked path
                    let mut relative_path = PathBuf::new();
                    
                    // Find which tracked path this file belongs to
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
                    
                    if let Some(base) = found_base {
                        if let Ok(rel) = file_path.strip_prefix(&base) {
                            relative_path = staging_dir.join("tracked").join(rel);
                        } else {
                            // Fallback: use filename
                            let file_name = file_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            relative_path = staging_dir.join("tracked").join(file_name);
                        }
                    } else {
                        // No base found, use filename
                        let file_name = file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        relative_path = staging_dir.join("tracked").join(file_name);
                    }
                    
                    // Ensure parent directories exist
                    if let Some(parent) = relative_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create staging directory: {}", e))?;
                    }
                    
                    // Copy file to staging
                    std::fs::copy(&file_path, &relative_path)
                        .map_err(|e| format!("Failed to copy file to staging: {}", e))?;
                    
                    relative_path
                };
                
                // Upload to Drive
                match drive_sync.upload_file(&staging_file, &folder_id).await {
                    Ok(_) => {
                        // Mark as synced
                        if let Err(e) = crate::file_tracker::mark_file_synced(&file_path) {
                            errors.push(format!("Failed to mark {} as synced: {}", 
                                file_path.display(), e));
                        }
                        files_synced += 1;
                    }
                    Err(e) => {
                        errors.push(format!("Failed to upload {}: {}", 
                            file_path.display(), e));
                    }
                }
            }
            Ok(false) => {
                // File hasn't changed, skip
                files_skipped += 1;
            }
            Err(e) => {
                errors.push(format!("Failed to check {}: {}", 
                    file_path.display(), e));
            }
        }
    }
    
    Ok(SyncResult {
        files_synced,
        files_skipped,
        errors,
    })
}

#[derive(Serialize)]
pub struct SyncStatus {
    last_sync: Option<u64>,
    next_sync: Option<u64>,
    is_syncing: bool,
}

#[tauri::command]
pub fn get_sync_status() -> Result<SyncStatus, String> {
    // Get last sync time from database
    // For now, return placeholder
    Ok(SyncStatus {
        last_sync: None,
        next_sync: None,
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

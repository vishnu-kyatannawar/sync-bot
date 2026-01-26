use tauri::AppHandle;
use tokio::time::{sleep, Duration};
use anyhow::Result;

pub async fn start_scheduler(app_handle: tauri::AppHandle) {
    let mut interval = Duration::from_secs(60); // Default 1 minute check interval
    
    loop {
        sleep(interval).await;
        
        // Check if auto-sync is enabled
        if let Ok(config) = crate::config::load_config() {
            if let Some(true) = config.auto_sync {
                if let Some(sync_interval) = config.sync_interval {
                    interval = Duration::from_secs(sync_interval * 60);
                    
                    // Perform sync
                    if let Err(e) = perform_scheduled_sync(&app_handle).await {
                        eprintln!("Scheduled sync error: {}", e);
                    }
                }
            } else {
                // Auto-sync disabled, check every minute
                interval = Duration::from_secs(60);
            }
        } else {
            interval = Duration::from_secs(60);
        }
    }
}

async fn perform_scheduled_sync(app_handle: &tauri::AppHandle) -> Result<()> {
    // This will be called by the scheduler
    // The actual sync logic is in commands.rs
    // We can emit an event to trigger sync from the frontend if needed
    app_handle.emit_all("scheduled-sync", ())?;
    Ok(())
}

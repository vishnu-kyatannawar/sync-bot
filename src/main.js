// Access Tauri APIs from the window object (since withGlobalTauri is enabled)
const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

const APP_LOG_PREFIX = '[Sync Bot]';

function logDebug(message, data = null) {
    const timestamp = new Date().toISOString();
    const logMessage = `${APP_LOG_PREFIX} [${timestamp}] ${message}`;
    if (data) {
        console.log(logMessage, data);
    } else {
        console.log(logMessage);
    }
}

function logError(message, error = null) {
    const timestamp = new Date().toISOString();
    const logMessage = `${APP_LOG_PREFIX} [ERROR] [${timestamp}] ${message}`;
    if (error) {
        console.error(logMessage, error);
    } else {
        console.error(logMessage);
    }
}

// UI Elements
const syncStatusEl = document.getElementById('sync-status');
const lastSyncEl = document.getElementById('last-sync');
const nextSyncEl = document.getElementById('next-sync');
const stagingDirEl = document.getElementById('staging-dir');
const driveFolderEl = document.getElementById('drive-folder');
const syncIntervalEl = document.getElementById('sync-interval');
const autoSyncEl = document.getElementById('auto-sync');
const fileListEl = document.getElementById('file-list');
const syncNowBtn = document.getElementById('sync-now');
const authenticateBtn = document.getElementById('authenticate');
const selectStagingDirBtn = document.getElementById('select-staging-dir');
const addFileBtn = document.getElementById('add-file');
const logOutputEl = document.getElementById('log-output');
const testJsBtn = document.getElementById('test-js');

logDebug('Main.js loading - global APIs initialized');

// Initialize
async function init() {
    logDebug('=== Application Initialization Started ===');
    
    // 1. ATTACH EVENT LISTENERS IMMEDIATELY
    // We do this first so the UI is responsive even if data loading takes time
    try {
        logDebug('Setting up event listeners (Priority 1)');
        setupEventListeners();
        logDebug('Event listeners attached successfully');
    } catch (error) {
        logError('CRITICAL: Failed to attach event listeners', error);
        log(`System Error: Could not initialize buttons. ${error}`, 'error');
    }

    try {
        log('Initializing application data...', 'info');
        
        // 2. Load configuration
        try {
            logDebug('Loading configuration from backend');
            const config = await invoke('get_config');
            logDebug('Configuration received', config);
            if (config) {
                if (config.staging_dir) stagingDirEl.value = config.staging_dir;
                if (config.drive_folder) driveFolderEl.value = config.drive_folder;
                if (config.sync_interval) syncIntervalEl.value = config.sync_interval;
                if (config.auto_sync !== undefined) autoSyncEl.checked = config.auto_sync;
                log('Configuration loaded', 'success');
            }
        } catch (e) {
            logError('Failed to load configuration', e);
            log(`Warning: Could not load saved settings.`, 'warning');
        }

        // 3. Load tracked files
        try {
            logDebug('Loading tracked files');
            await loadTrackedFiles();
        } catch (e) {
            logError('Failed to load tracked files', e);
        }

        // 4. Load status
        try {
            logDebug('Loading sync status');
            await updateStatus();
        } catch (e) {
            logDebug('Initial status check failed (normal if first run)');
        }

        log('Application ready', 'success');
        logDebug('=== Application Initialization Completed ===');
    } catch (error) {
        logError('General initialization error', error);
        log(`Initialization error: ${error}`, 'error');
    }
}

function setupEventListeners() {
    logDebug('Entering setupEventListeners()');
    
    if (!selectStagingDirBtn) {
        logError('CRITICAL: selectStagingDirBtn (Browse) NOT FOUND IN DOM');
        return;
    }

    if (testJsBtn) {
        testJsBtn.addEventListener('click', () => {
            console.log('Test UI button clicked');
            alert('UI Interaction working! JavaScript is active.');
            log('UI Test Successful', 'success');
        });
    }

    selectStagingDirBtn.addEventListener('click', async () => {
        logDebug('=== Browse Button Click Handler Started ===');
        try {
            logDebug('Browse button clicked - event handler triggered');
            log('Opening directory selector...', 'info');
            
            logDebug('Calling dialog.open() with options', {
                directory: true,
                multiple: false,
                defaultPath: stagingDirEl.value || undefined
            });
            
            const selected = await open({
                directory: true,
                multiple: false,
                defaultPath: stagingDirEl.value || undefined
            });
            
            logDebug('Dialog.open() returned', { selected });
            console.log('Selected directory:', selected);
            
            if (selected) {
                logDebug('Directory was selected, updating UI and config', { path: selected });
                stagingDirEl.value = selected;
                
                logDebug('Invoking set_staging_dir command');
                await invoke('set_staging_dir', { path: selected });
                
                log(`Staging directory set to: ${selected}`, 'success');
                logDebug('set_staging_dir command completed successfully');
            } else {
                logDebug('No directory selected (user cancelled)');
                log('No directory selected', 'info');
            }
        } catch (error) {
            logError('Error in Browse button handler', error);
            console.error('Error selecting staging directory:', error);
            console.error('Error stack:', error.stack);
            log(`Error selecting staging directory: ${error}`, 'error');
        }
        logDebug('=== Browse Button Click Handler Ended ===');
    });

    addFileBtn.addEventListener('click', async () => {
        try {
            const selected = await open({
                directory: false,
                multiple: true
            });
            if (selected && selected.length > 0) {
                for (const path of selected) {
                    await invoke('add_tracked_path', { path });
                }
                await loadTrackedFiles();
                log(`Added ${selected.length} file(s)/folder(s)`, 'success');
            }
        } catch (error) {
            log(`Error adding files: ${error}`, 'error');
        }
    });

    syncNowBtn.addEventListener('click', async () => {
        try {
            syncNowBtn.disabled = true;
            syncStatusEl.textContent = 'Syncing...';
            syncStatusEl.className = 'status-value syncing';
            log('Starting sync...', 'info');

            const result = await invoke('sync_now');
            log(`Sync completed: ${result.files_synced} files synced`, 'success');
            syncStatusEl.textContent = 'Sync Complete';
            syncStatusEl.className = 'status-value success';
            await updateStatus();
        } catch (error) {
            log(`Sync error: ${error}`, 'error');
            syncStatusEl.textContent = 'Sync Failed';
            syncStatusEl.className = 'status-value error';
        } finally {
            syncNowBtn.disabled = false;
        }
    });

    authenticateBtn.addEventListener('click', async () => {
        try {
            authenticateBtn.disabled = true;
            log('Starting Google Drive authentication...', 'info');
            const url = await invoke('get_auth_url');
            // Open browser for OAuth
            await invoke('open_url', { url });
            log('Please complete authentication in the browser', 'info');
        } catch (error) {
            log(`Authentication error: ${error}`, 'error');
        } finally {
            authenticateBtn.disabled = false;
        }
    });

    // Save config on change
    driveFolderEl.addEventListener('change', async () => {
        await invoke('set_drive_folder', { folder: driveFolderEl.value });
    });

    syncIntervalEl.addEventListener('change', async () => {
        await invoke('set_sync_interval', { interval: parseInt(syncIntervalEl.value) });
    });

    autoSyncEl.addEventListener('change', async () => {
        await invoke('set_auto_sync', { enabled: autoSyncEl.checked });
    });
}

async function loadTrackedFiles() {
    try {
        const files = await invoke('get_tracked_paths');
        fileListEl.innerHTML = '';
        
        if (files.length === 0) {
            fileListEl.innerHTML = '<div class="empty-state"><p>No files or folders tracked yet.</p><p>Click "Add File/Folder" to get started.</p></div>';
            return;
        }

        for (const file of files) {
            const item = document.createElement('div');
            item.className = 'file-item';
            item.innerHTML = `
                <span class="file-path">${file}</span>
                <button class="file-remove" data-path="${file}">Remove</button>
            `;
            fileListEl.appendChild(item);

            item.querySelector('.file-remove').addEventListener('click', async () => {
                try {
                    await invoke('remove_tracked_path', { path: file });
                    await loadTrackedFiles();
                    log(`Removed: ${file}`, 'info');
                } catch (error) {
                    log(`Error removing file: ${error}`, 'error');
                }
            });
        }
    } catch (error) {
        log(`Error loading tracked files: ${error}`, 'error');
    }
}

async function updateStatus() {
    try {
        const status = await invoke('get_sync_status');
        if (status.last_sync) {
            lastSyncEl.textContent = new Date(status.last_sync * 1000).toLocaleString();
        }
        if (status.next_sync) {
            nextSyncEl.textContent = new Date(status.next_sync * 1000).toLocaleString();
        }
    } catch (error) {
        // Ignore status errors
    }
}

function log(message, type = 'info') {
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    logOutputEl.appendChild(entry);
    logOutputEl.scrollTop = logOutputEl.scrollHeight;
}

// Initialize on load
logDebug('Document loaded, starting initialization');
init();

// Update status periodically
setInterval(updateStatus, 30000); // Every 30 seconds

// Log when the window is fully loaded
window.addEventListener('load', () => {
    logDebug('Window fully loaded');
});

// Log any unhandled errors
window.addEventListener('error', (event) => {
    logError('Unhandled error', {
        message: event.message,
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
        error: event.error
    });
});

// Log any unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
    logError('Unhandled promise rejection', {
        reason: event.reason,
        promise: event.promise
    });
});

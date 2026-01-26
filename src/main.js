import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

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

// Initialize
async function init() {
    try {
        // Load configuration
        const config = await invoke('get_config');
        if (config) {
            if (config.staging_dir) stagingDirEl.value = config.staging_dir;
            if (config.drive_folder) driveFolderEl.value = config.drive_folder;
            if (config.sync_interval) syncIntervalEl.value = config.sync_interval;
            if (config.auto_sync !== undefined) autoSyncEl.checked = config.auto_sync;
        }

        // Load tracked files
        await loadTrackedFiles();

        // Load status
        await updateStatus();

        // Set up event listeners
        setupEventListeners();

        log('Application initialized', 'info');
    } catch (error) {
        log(`Initialization error: ${error}`, 'error');
    }
}

function setupEventListeners() {
    selectStagingDirBtn.addEventListener('click', async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                defaultPath: stagingDirEl.value || undefined
            });
            if (selected) {
                stagingDirEl.value = selected;
                await invoke('set_staging_dir', { path: selected });
                log(`Staging directory set to: ${selected}`, 'info');
            }
        } catch (error) {
            log(`Error selecting staging directory: ${error}`, 'error');
        }
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
init();

// Update status periodically
setInterval(updateStatus, 30000); // Every 30 seconds

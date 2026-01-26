# Testing Instructions - Browse Button Fix

## What I've Added

### 1. **Comprehensive Logging System**

#### Backend Logs (Rust)
- Location: `/home/vishnu/projects/personal/sync-bot/logs/sync-bot_YYYY-MM-DD_HH-MM-SS.log`
- Logs all backend operations including:
  - Application startup
  - Configuration loading
  - Database initialization
  - All command invocations
  - Errors and warnings

#### Frontend Logs (JavaScript)
- Location: Browser Developer Console
- Logs all frontend operations including:
  - Application initialization
  - Button clicks
  - Dialog interactions
  - Errors and promise rejections

### 2. **Enhanced Debug Information**
- Every button click is now logged with timestamps
- Dialog plugin calls are logged with parameters
- Success/failure of operations is logged

## How to Test the Browse Button

### Step 1: Start the Application

**Run as your normal user (NOT as root):**

```bash
cd /home/vishnu/projects/personal/sync-bot
npm run dev
```

The application should open in a new window.

### Step 2: Open Developer Tools

1. When the app window opens, right-click anywhere
2. Select **"Inspect"** or **"Inspect Element"**
3. Click on the **"Console"** tab
4. You should see logs like:
   ```
   [Sync Bot] [timestamp] Main.js loaded
   [Sync Bot] [timestamp] Document loaded, starting initialization
   [Sync Bot] [timestamp] === Application Initialization Started ===
   ```

### Step 3: Monitor Logs

Open a terminal and watch the backend logs in real-time:

```bash
cd /home/vishnu/projects/personal/sync-bot
./view-logs.sh
```

Or manually:
```bash
tail -f logs/sync-bot_*.log
```

You should see:
```
[timestamp] [INFO] === Sync Bot Started ===
[timestamp] [INFO] Log file: "/home/vishnu/projects/personal/sync-bot/logs/sync-bot_..."
[timestamp] [INFO] Starting Sync Bot application...
[timestamp] [INFO] Initializing Tauri plugins...
```

### Step 4: Click the Browse Button

1. In the app window, find the **"Staging Directory"** section
2. Click the **"Browse"** button
3. Watch BOTH:
   - **Browser Console** for frontend logs
   - **Terminal** (running view-logs.sh) for backend logs

### Expected Behavior

#### In Browser Console:
```
[Sync Bot] [timestamp] === Browse Button Click Handler Started ===
[Sync Bot] [timestamp] Browse button clicked - event handler triggered
[Sync Bot] [timestamp] Calling dialog.open() with options {directory: true, multiple: false}
[Sync Bot] [timestamp] Dialog.open() returned {selected: "/path/to/directory"}
[Sync Bot] [timestamp] Directory was selected, updating UI and config {path: "/path/to/directory"}
[Sync Bot] [timestamp] Invoking set_staging_dir command
[Sync Bot] [timestamp] set_staging_dir command completed successfully
[Sync Bot] [timestamp] === Browse Button Click Handler Ended ===
```

#### In Backend Logs:
```
[timestamp] [INFO] Command: set_staging_dir called with path: /path/to/directory
[timestamp] [INFO] set_staging_dir completed successfully
```

#### In the UI:
- A native file selection dialog should appear
- After selecting a directory, the path should appear in the input field
- The log panel should show: "Staging directory set to: /path/to/directory"

## Troubleshooting

### Issue 1: Button Doesn't Respond

**Symptoms:** No logs appear when clicking the button

**Check:**
1. Is the app running? `ps aux | grep sync-bot`
2. Are there any JavaScript errors in the console?
3. Is the event listener attached? Look for "Event listeners set up" in console

**Solution:** Refresh the app or restart it

### Issue 2: Dialog Doesn't Open

**Symptoms:** Logs show "Browse button clicked" but no dialog appears

**Check Browser Console for errors like:**
- `Failed to resolve module specifier '@tauri-apps/plugin-dialog'`
- `dialog is not defined`
- `Permission denied`

**Possible Causes:**
1. **Plugin not installed**: 
   ```bash
   ls node_modules/@tauri-apps/plugin-dialog/
   ```
   If not found:
   ```bash
   npm install
   ```

2. **Permissions not configured**: Check `src-tauri/capabilities/default.json` contains:
   ```json
   "dialog:allow-open"
   ```

3. **Running as root**: Don't run as root! Use your normal user account.

### Issue 3: Dialog Opens But Selection Doesn't Save

**Symptoms:** Dialog opens, you select a directory, but nothing happens

**Check Backend Logs for:**
- `Command: set_staging_dir called with path: ...`
- Any errors after this line

**Possible Causes:**
1. Configuration file permissions
2. Backend command not registered
3. Path validation failing

**Solution:** Check the full error in backend logs and report it

### Issue 4: App Fails to Start

**Symptoms:** `npm run dev` hangs or shows GTK errors

**GTK Error Example:**
```
Failed to initialize GTK backend!
```

**Solution:** 
- Make sure you're running from a desktop environment (not SSH)
- Make sure you're NOT running as root
- Try: `unset DISPLAY && export DISPLAY=:0`

## Getting Detailed Debug Info

If the issue persists, collect this information:

### 1. Full Backend Log
```bash
cat logs/sync-bot_*.log | tail -200 > debug-backend.log
```

### 2. Browser Console Export
1. Right-click in the console
2. Select "Save as..."
3. Save to `debug-console.log`

### 3. System Information
```bash
echo "OS: $(lsb_release -d)" > debug-system.txt
echo "Node: $(node --version)" >> debug-system.txt
echo "Rustc: $(rustc --version)" >> debug-system.txt
echo "User: $(whoami)" >> debug-system.txt
echo "Display: $DISPLAY" >> debug-system.txt
```

### 4. Plugin Check
```bash
echo "=== NPM Packages ===" > debug-plugins.txt
npm list @tauri-apps/plugin-dialog @tauri-apps/plugin-fs @tauri-apps/plugin-shell >> debug-plugins.txt
echo -e "\n=== Cargo Dependencies ===" >> debug-plugins.txt
grep -A 2 "tauri-plugin" src-tauri/Cargo.toml >> debug-plugins.txt
```

Then share:
- `debug-backend.log`
- `debug-console.log`
- `debug-system.txt`
- `debug-plugins.txt`

## Quick Commands Reference

```bash
# Start app
npm run dev

# View logs
./view-logs.sh

# Check if app is running
ps aux | grep sync-bot

# Kill all instances
pkill -f sync-bot

# Clean build
cd src-tauri && cargo clean && cd .. && npm run dev

# Re-install dependencies
npm install

# View latest 50 log lines
tail -50 logs/sync-bot_*.log
```

## Next Steps

After testing the Browse button:
1. Test the "Add File/Folder" button (should also open a file dialog)
2. Test the "Authenticate Google Drive" button
3. Test the "Sync Now" button

Each button now has extensive logging to help debug any issues!

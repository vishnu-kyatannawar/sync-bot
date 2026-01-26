# Debugging Guide for Sync Bot

## How to Debug the Browse Button Issue

### 1. Check Application Logs

The application now generates detailed logs in the `logs/` directory.

```bash
# View the latest log file in real-time:
./view-logs.sh

# Or manually:
tail -f logs/sync-bot_*.log
```

### 2. Check Browser Console

When the app is running:
1. Open the app window
2. Right-click anywhere in the window
3. Select "Inspect" or "Inspect Element"
4. Go to the "Console" tab
5. Look for messages starting with `[Sync Bot]`

When you click the Browse button, you should see:
```
[Sync Bot] [timestamp] === Browse Button Click Handler Started ===
[Sync Bot] [timestamp] Browse button clicked - event handler triggered
[Sync Bot] [timestamp] Calling dialog.open() with options
```

### 3. What to Look For

#### If the button doesn't respond at all:
- Check if you see "Browse button clicked" in the console
- If not, the event listener might not be attached
- Look for JavaScript errors in the console

#### If the button responds but dialog doesn't open:
- Look for errors related to `@tauri-apps/plugin-dialog`
- Check if you see "Dialog.open() returned" in the console
- Look for permission errors in the Rust logs

#### If the dialog opens but selection doesn't save:
- Look for "set_staging_dir command completed successfully" in the console
- Check the Rust logs for "Command: set_staging_dir called"

### 4. Common Issues and Solutions

#### Issue: "Plugin not found" or "Command not available"
**Solution**: Ensure the dialog plugin is properly initialized in Rust:
```rust
.plugin(tauri_plugin_dialog::init())
```

#### Issue: "Permission denied" in logs
**Solution**: Check `src-tauri/capabilities/default.json` has:
```json
"dialog:allow-open"
```

#### Issue: Nothing in the logs
**Solution**: 
1. Make sure you're running the app as your normal user (not root)
2. Check if logs directory exists: `ls -la logs/`
3. Verify the app is actually running: `ps aux | grep sync-bot`

### 5. Manual Testing

You can test the dialog plugin from the browser console:

```javascript
// Open the developer console and try:
import('@tauri-apps/plugin-dialog').then(module => {
    module.open({
        directory: true,
        multiple: false
    }).then(result => {
        console.log('Dialog result:', result);
    }).catch(err => {
        console.error('Dialog error:', err);
    });
});
```

### 6. Verify Tauri Plugin Installation

Check if the dialog plugin is installed:
```bash
# Check Cargo.toml
grep "tauri-plugin-dialog" src-tauri/Cargo.toml

# Check if it's in node_modules
ls -la node_modules/@tauri-apps/plugin-dialog/
```

### 7. Re-install Plugins if Needed

If plugins seem to be missing:
```bash
# Re-install npm packages
npm install

# Clean and rebuild
cd src-tauri
cargo clean
cd ..
npm run dev
```

## Getting Help

When reporting issues, please include:
1. Full contents of the latest log file from `logs/`
2. Browser console output (copy all messages)
3. Your operating system and version
4. Node.js version: `node --version`
5. Rust version: `rustc --version`

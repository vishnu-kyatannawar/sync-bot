# Release Notes - v2.0.0

## What's New

### 🎨 UI/UX Improvements
- **Compact Professional Layout**: Redesigned UI with two-column grid layout for better space utilization
- **Removed Icons**: Cleaner, professional interface without emoji icons
- **Version Display**: App version now visible in the header
- **Improved Status Indicators**: Better visual feedback for sync operations

### 🔧 Core Functionality Enhancements
- **System Tray Support**: App now runs in the system tray with background operation
  - Minimize to tray instead of closing
  - Tray menu with Show Window, Sync Now, and Quit options
  - Left-click tray icon to toggle window visibility
- **Automatic Token Refresh**: Fixed 401 authentication errors with automatic OAuth token renewal
- **Retry Logic**: All Google Drive API operations now retry up to 3 times on failure
- **Smart Sync Status Tracking**: Last Sync and Next Sync times now display correctly

### 🐛 Bug Fixes
- **Fixed Folder Structure in ZIP**: Removed duplicate folder names, preserves original hierarchy
- **Permission Handling**: Fixed "Permission denied" errors when copying read-only files
- **Google OAuth Migration**: Migrated from deprecated OOB flow to modern Loopback IP authentication
- **Token Expiration**: Automatic refresh when access tokens expire (no more hourly failures)
- **ZIP File Structure**: Removed unnecessary "tracked" wrapper folder

### 📦 Infrastructure
- **Build Scripts**: Added clean-build.sh and refresh-icon.sh utilities
- **Enhanced Logging**: Improved log messages with retry attempt counters
- **Documentation**: Added FORCE_PUSH_INSTRUCTIONS.md and updated release guides

## Breaking Changes
None. Existing configurations and data are compatible.

## Upgrade Notes
- System tray requires the `tray-icon` Tauri feature (already included)
- Icon refresh requires clearing build cache (use `./refresh-icon.sh`)

## Known Issues
- None at this time

## Installation

Download `sync-bot_2.0.0_amd64.AppImage` from the release assets.

```bash
chmod +x sync-bot_2.0.0_amd64.AppImage
./sync-bot_2.0.0_amd64.AppImage
```

## Full Changelog

```
a216921 chore: add icon refresh script and improve build cache management
a82f27f feat: add system tray support for background operation
25ecde6 fix: refactor retry logic to use loops instead of async recursion
80523b7 fix: implement automatic token refresh and retry logic
9c9acf0 fix: remove AppImage from git tracking, add to .gitignore
6367a4f fix: preserve tracked folder names in ZIP structure
600554f fix: remove unnecessary 'tracked' folder from ZIP structure
4afd1ed refactor: redesign UI for compact professional layout
b5cf693 chore: remove Quick Help section from UI
d6e29e3 fix: sync status tracking and remove test UI button
86929b0 feat: enhanced sync logic, improved UI, and automated Google OAuth
e3543f9 fix: UI responsiveness and enhanced logging system
```

## Contributors
- vishnu

---
**Full Documentation**: See README.md and BUILD.md for setup instructions.

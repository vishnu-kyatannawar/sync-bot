# Sync Bot Releases

This directory contains AppImage files for each release version.

**Important**: AppImage files are NOT committed to the repository (they exceed GitHub's 100MB limit). They are stored locally and should be uploaded directly to GitHub releases.

## Available Versions

### v2.0.0 (Latest)
- **File**: `sync-bot_2.0.0_amd64.AppImage`
- **Size**: ~102 MB
- **Changes**: System tray support, automatic token refresh, retry logic, compact UI
- **Release Notes**: See `RELEASE_NOTES_v2.0.0.md`

### v1.0.0
- **File**: `sync-bot_1.0.0_amd64.AppImage`
- **Size**: ~102 MB
- **Changes**: Initial release with ZIP-based sync, OAuth authentication
- **Release Notes**: See `GITHUB_RELEASE.md`

## Installation

1. Download the AppImage file
2. Make it executable: `chmod +x sync-bot_X.X.X_amd64.AppImage`
3. Run: `./sync-bot_X.X.X_amd64.AppImage`

## Building a Release

```bash
# Clean build for new version
npm run build

# AppImage will be created at:
# src-tauri/target/release/bundle/appimage/Sync Bot_X.X.X_amd64.AppImage

# Copy to releases folder
mkdir -p releases/vX.X.X
cp "src-tauri/target/release/bundle/appimage/Sync Bot_X.X.X_amd64.AppImage" \
   "releases/vX.X.X/sync-bot_X.X.X_amd64.AppImage"
```

**Note**: Files in this directory are ignored by git (see `.gitignore`).

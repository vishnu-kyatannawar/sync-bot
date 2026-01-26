# Creating GitHub Release for v1.0.0

## Step 1: Push the Tag

```bash
git push origin v1.0.0
```

## Step 2: Create GitHub Release

1. Go to your GitHub repository: `https://github.com/YOUR_USERNAME/sync-bot`
2. Click on **"Releases"** (in the right sidebar or under "Code")
3. Click **"Draft a new release"**
4. **Select tag**: Choose `v1.0.0` from the dropdown
5. **Release title**: `v1.0.0`
6. **Description**: Copy and paste the release notes below
7. **Attach binaries**: 
   - Click "Attach binaries by dropping them here or selecting them"
   - Upload: `releases/v1.0.0/sync-bot_1.0.0_amd64.AppImage`
8. Click **"Publish release"**

## Release Notes

```markdown
# Sync Bot v1.0.0

## 🎉 First Release

Sync Bot is a lightweight, intelligent desktop application for syncing files and folders to Google Drive with smart change detection.

## ✨ Features

- **ZIP-based Sync**: Creates a single `backup.zip` file containing all your tracked files
- **Smart Change Detection**: Only syncs files that have actually changed using SHA256 hashing
- **Automated Authentication**: Modern OAuth flow with automatic code capture (no copy-paste needed)
- **Recursive Folder Tracking**: Track entire directories including hidden files and subfolders
- **Version History**: Automatically keeps the last 4 sync copies as timestamped zip archives
- **Compact Professional UI**: Clean, information-dense interface that fits everything on one screen
- **Comprehensive Logging**: Detailed logs for troubleshooting in `logs/` directory
- **Permission Handling**: Automatically handles read-only files (SSH keys, config files, etc.)

## 📦 Installation

1. Download `sync-bot_1.0.0_amd64.AppImage`
2. Make it executable: `chmod +x sync-bot_1.0.0_amd64.AppImage`
3. Run: `./sync-bot_1.0.0_amd64.AppImage`

## 🔧 Requirements

- Linux (tested on Debian-based systems)
- Google Cloud Console account (free) for OAuth credentials
- Google Drive account

## 📝 First Time Setup

1. Get your Google Client ID and Secret from [Google Cloud Console](https://console.cloud.google.com/)
2. Enter them in the app's Configuration section
3. Click "Authenticate" to connect your Google Drive
4. Select a staging directory
5. Add files/folders to track
6. Click "Sync Now"!

## 🐛 Known Issues

None at this time.

## 📄 License

MIT
```

## File Location

The AppImage is ready at:
- **Path**: `releases/v1.0.0/sync-bot_1.0.0_amd64.AppImage`
- **Size**: Check with `ls -lh releases/v1.0.0/`

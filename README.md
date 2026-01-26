# Sync Bot

A lightweight desktop application for syncing files and folders to Google Drive with smart change detection and version management.

## Features

- **Smart Sync**: Only syncs files that have actually changed (SHA256 hash comparison)
- **Periodic Sync**: Automatic syncing at configurable intervals
- **Version History**: Keeps the last 4 sync copies as timestamped zip archives
- **Lightweight**: Built with Tauri for minimal resource usage (~30-50MB RAM)
- **Cross-Platform**: Works on all Linux distributions
- **AppImage**: Can be packaged as a standalone AppImage

## Requirements

- Rust 1.93.0+ (for building) - Install from https://rustup.rs/
- Node.js 24+ and npm - Install from https://nodejs.org/
- System dependencies (see BUILD.md for details)
- Google Cloud Project with Drive API enabled
- Tauri 2.0

## Quick Start

1. Install dependencies:
```bash
npm install
```

2. Set up Google Drive OAuth credentials (see BUILD.md)

3. Run in development mode:
```bash
npm run dev
```

4. Build AppImage:
```bash
npm run build
```

For detailed build instructions, see [BUILD.md](BUILD.md).

## Configuration

The application stores configuration in:
- Config file: `~/.config/sync-bot/config.toml`
- Staging directory: `~/.local/share/sync-bot/staging/` (or set via `SYNC_BOT_STAGING_DIR` env var)
- Archives: `~/.local/share/sync-bot/archives/`
- Database: `~/.local/share/sync-bot/sync_bot.db`

## Google Drive Setup

1. Create a project in [Google Cloud Console](https://console.cloud.google.com/)
2. Enable Google Drive API
3. Create OAuth 2.0 credentials
4. Add the credentials to the application (via the authentication flow)

## Usage

1. Launch the application
2. Click "Authenticate Google Drive" to set up OAuth
3. Select a staging directory (or use default)
4. Add files/folders to track
5. Configure sync interval and enable auto-sync if desired
6. Click "Sync Now" to perform manual sync

## License

MIT

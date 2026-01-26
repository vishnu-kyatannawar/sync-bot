# Release v1.0.0

## Building the Release

To build the AppImage for Linux:

```bash
# Make sure you're in the project directory
cd /home/vishnu/projects/personal/sync-bot

# Build the release
CI=false npm run build

# The AppImage will be created at:
# src-tauri/target/release/bundle/appimage/sync-bot_*.AppImage
```

## Creating the Release Tag

The git tag has been created. To push it:

```bash
git push origin v1.0.0
```

## Release Files

After building, you'll find the AppImage at:
- `src-tauri/target/release/bundle/appimage/sync-bot_*.AppImage`

## GitHub Release

To create a GitHub release:

1. Go to your repository on GitHub
2. Click "Releases" → "Draft a new release"
3. Select tag: `v1.0.0`
4. Upload the AppImage file
5. Add release notes and publish

## Version Display

The version (v1.0.0) is now displayed in the application header.

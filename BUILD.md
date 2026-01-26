# Build Instructions

## Prerequisites

1. **Rust**: Install from https://rustup.rs/ (version 1.93.0 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js and npm**: Install from https://nodejs.org/ (Node 24 or later)

3. **System Dependencies** (for Linux):
   ```bash
   # Debian/Ubuntu
   sudo apt-get update
   sudo apt-get install libwebkit2gtk-4.1-dev \
       build-essential \
       curl \
       wget \
       libssl-dev \
       libgtk-3-dev \
       libayatana-appindicator3-dev \
       librsvg2-dev
   ```
   
   Note: Tauri 2.0 requires WebKitGTK 4.1 or later

## Development Setup

1. Install npm dependencies:
   ```bash
   npm install
   ```

2. Set up Google Drive OAuth credentials:
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or select existing
   - Enable Google Drive API
   - Create OAuth 2.0 credentials (Desktop app)
   - Set environment variables:
     ```bash
     export GOOGLE_CLIENT_ID="your_client_id"
     export GOOGLE_CLIENT_SECRET="your_client_secret"
     ```

3. Run in development mode:
   ```bash
   npm run dev
   ```

## Building AppImage

1. Build the application:
   ```bash
   npm run build
   ```

2. The AppImage will be generated in:
   ```
   src-tauri/target/release/bundle/appimage/
   ```

3. Make it executable and run:
   ```bash
   chmod +x src-tauri/target/release/bundle/appimage/sync-bot_*.AppImage
   ./src-tauri/target/release/bundle/appimage/sync-bot_*.AppImage
   ```

## Creating Icons

You'll need to create icon files for the application. You can use tools like:
- GIMP
- ImageMagick
- Online icon generators

Required icon files:
- `src-tauri/icons/32x32.png` - 32x32 pixels
- `src-tauri/icons/128x128.png` - 128x128 pixels
- `src-tauri/icons/128x128@2x.png` - 256x256 pixels (for high DPI)
- `src-tauri/icons/icon.icns` - macOS icon (optional)
- `src-tauri/icons/icon.ico` - Windows icon (optional)

## Troubleshooting

### Build fails with missing dependencies
- Make sure all system dependencies are installed
- On Debian/Ubuntu, install the packages listed above

### OAuth not working
- Verify GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET are set
- Check that Google Drive API is enabled in your project
- Ensure OAuth consent screen is configured

### AppImage not launching
- Check file permissions: `chmod +x *.AppImage`
- Try running from terminal to see error messages
- Verify all dependencies are available on the target system

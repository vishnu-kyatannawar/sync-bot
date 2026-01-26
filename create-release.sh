#!/bin/bash

set -e

VERSION="1.0.0"
TAG_NAME="v${VERSION}"

echo "=== Sync Bot Release Script ==="
echo "Version: ${VERSION}"
echo ""

# Check if build exists
APPIMAGE_PATH=$(find src-tauri/target/release/bundle/appimage -name "*.AppImage" 2>/dev/null | head -1)

if [ -z "$APPIMAGE_PATH" ]; then
    echo "❌ AppImage not found. Building now..."
    echo "This may take 10-30 minutes..."
    CI=false npm run build
    
    APPIMAGE_PATH=$(find src-tauri/target/release/bundle/appimage -name "*.AppImage" 2>/dev/null | head -1)
    
    if [ -z "$APPIMAGE_PATH" ]; then
        echo "❌ Build failed or AppImage not found"
        exit 1
    fi
fi

echo "✅ Found AppImage: $APPIMAGE_PATH"
APPIMAGE_NAME=$(basename "$APPIMAGE_PATH")

# Create release directory
RELEASE_DIR="releases/v${VERSION}"
mkdir -p "$RELEASE_DIR"

# Copy AppImage to release directory with a clean name
RELEASE_APPIMAGE_NAME="sync-bot_${VERSION}_amd64.AppImage"
cp "$APPIMAGE_PATH" "$RELEASE_DIR/$RELEASE_APPIMAGE_NAME"

echo "✅ Copied AppImage to $RELEASE_DIR/$RELEASE_APPIMAGE_NAME"

# Check if tag already exists
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo "⚠️  Tag $TAG_NAME already exists. Deleting and recreating..."
    git tag -d "$TAG_NAME" 2>/dev/null || true
    git push origin ":refs/tags/$TAG_NAME" 2>/dev/null || true
fi

# Create git tag
echo "Creating git tag: $TAG_NAME"
git tag -a "$TAG_NAME" -m "Release version ${VERSION}

- ZIP-based sync with smart change detection
- Automated Google OAuth authentication
- Recursive folder tracking with hidden files support
- Version history (last 4 syncs)
- Compact professional UI"

echo "✅ Created tag: $TAG_NAME"

# Show instructions
echo ""
echo "=== Release Created Successfully ==="
echo "Tag: $TAG_NAME"
echo "AppImage: $RELEASE_DIR/$APPIMAGE_NAME"
echo ""
echo "To push the tag to remote:"
echo "  git push origin $TAG_NAME"
echo ""
echo "To create a GitHub release:"
echo "  1. Go to: https://github.com/YOUR_USERNAME/sync-bot/releases/new"
echo "  2. Select tag: $TAG_NAME"
echo "  3. Upload: $RELEASE_DIR/$APPIMAGE_NAME"
echo "  4. Add release notes and publish"
echo ""

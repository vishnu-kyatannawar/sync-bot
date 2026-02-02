#!/bin/bash

echo "=== Sync Bot Icon Refresh ==="
echo ""

# Kill any running instances
pkill -f "sync-bot" 2>/dev/null || true
pkill -f "tauri dev" 2>/dev/null || true

echo "1. Cleaning build cache..."
cd src-tauri
rm -rf target/debug target/release 2>/dev/null || true
rm -rf gen/ 2>/dev/null || true
cd ..

echo "2. Icon file info:"
ls -lh src-tauri/icons/icon.png

echo ""
echo "3. Rebuilding with new icon..."
echo "   This will take a moment..."
npm run dev

echo ""
echo "✅ Icon refreshed. The app should now use the new icon."

#!/bin/bash

echo "Cleaning up build processes and locks..."

# Kill any running build processes
pkill -f "tauri dev" 2>/dev/null || true
pkill -f "tauri build" 2>/dev/null || true
pkill -f "cargo build" 2>/dev/null || true

# Wait a moment for processes to die
sleep 2

# Remove Cargo lock files (they'll be regenerated)
find src-tauri/target -name "*.lock" -type f -delete 2>/dev/null || true

echo "✅ Cleanup complete. You can now run: npm run build"

#!/bin/bash

# Script to view Sync Bot logs

LOGS_DIR="/home/vishnu/projects/personal/sync-bot/logs"

echo "=== Sync Bot Log Viewer ==="
echo ""

if [ ! -d "$LOGS_DIR" ]; then
    echo "Error: Logs directory not found at $LOGS_DIR"
    exit 1
fi

# Get the latest log file
LATEST_LOG=$(ls -t "$LOGS_DIR"/sync-bot_*.log 2>/dev/null | head -1)

if [ -z "$LATEST_LOG" ]; then
    echo "No log files found in $LOGS_DIR"
    echo "Run the application first to generate logs."
    exit 1
fi

echo "Viewing latest log file: $(basename "$LATEST_LOG")"
echo "Press Ctrl+C to exit"
echo "========================================"
echo ""

# Follow the log file
tail -f "$LATEST_LOG"

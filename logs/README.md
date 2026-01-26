# Application Logs

This directory contains application logs for debugging and monitoring purposes.

## Log Files

- **Format**: `sync-bot_YYYY-MM-DD_HH-MM-SS.log`
- **Content**: Detailed logs from both Rust backend and JavaScript frontend
- **Location**: `/home/vishnu/projects/personal/sync-bot/logs/`

## Log Levels

- **INFO**: General information about application flow
- **WARN**: Warning messages for non-critical issues
- **ERROR**: Error messages for failures
- **DEBUG**: Detailed debugging information

## Viewing Logs

To view the latest log file:
```bash
tail -f logs/sync-bot_*.log
```

To view all logs from the latest file:
```bash
cat logs/sync-bot_*.log | tail -100
```

## Cleanup

Logs are not automatically cleaned up. To remove old logs:
```bash
rm logs/sync-bot_*.log
```

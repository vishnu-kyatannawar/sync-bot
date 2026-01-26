# Quick Start - Testing the Browse Button

## ✅ Logging System is Now Active!

I've added comprehensive logging to help debug the Browse button issue.

## 🚀 How to Test

### Option 1: Run with Log Viewer (Recommended)

**Terminal 1 - Start the app:**
```bash
cd /home/vishnu/projects/personal/sync-bot
npm run dev
```

**Terminal 2 - Watch logs:**
```bash
cd /home/vishnu/projects/personal/sync-bot
./view-logs.sh
```

### Option 2: Run and Check Logs Later

```bash
cd /home/vishnu/projects/personal/sync-bot
npm run dev

# In another terminal, view logs:
cat logs/sync-bot_*.log
```

## 🔍 What to Look For

### When you click the Browse button:

**✅ If it works, you'll see:**

**Browser Console (F12 → Console tab):**
```
[Sync Bot] === Browse Button Click Handler Started ===
[Sync Bot] Browse button clicked - event handler triggered
[Sync Bot] Calling dialog.open() with options
[Sync Bot] Dialog.open() returned
[Sync Bot] Directory was selected
[Sync Bot] set_staging_dir command completed successfully
```

**Backend Log file:**
```
[INFO] Command: set_staging_dir called with path: /your/selected/path
[INFO] set_staging_dir completed successfully
```

**❌ If it doesn't work, you'll see:**
- Errors in the browser console
- Error messages in the log file
- Missing log entries (indicating where it fails)

## 📋 Current Logs

Check the `logs/` directory:
```bash
ls -lah logs/
```

Latest log file:
```bash
tail -50 logs/sync-bot_*.log
```

## 🐛 If Browse Button Still Doesn't Work

1. **Open browser console** (Right-click → Inspect → Console tab)
2. **Click Browse button**
3. **Copy ALL console output**
4. **Copy the log file**:
   ```bash
   cat logs/sync-bot_*.log > browse-button-debug.log
   ```
5. **Share both** so we can see exactly what's happening

## 📚 More Information

- **Full testing guide**: See `TESTING_INSTRUCTIONS.md`
- **Debug guide**: See `DEBUG_GUIDE.md`
- **Log viewer script**: `./view-logs.sh`

## 🎯 Important Notes

- **Run as normal user** (not root)
- **Check permissions** on capabilities file
- **Ensure plugins are installed**: `npm install`
- **Browser console is key** - it shows JavaScript errors

---

**The app is ready to run. Start it with `npm run dev` and test the Browse button!**

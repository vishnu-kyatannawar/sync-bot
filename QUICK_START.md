# Quick Start - Sync Bot

## ✅ System Ready!

I've fixed the Google Authentication error and automated the process.

## 🚀 How to Test Authentication

### 1. Start the App
```bash
npm run dev
```

### 2. Enter Credentials
- Go to the **Google Client ID** and **Secret** fields in the UI.
- Paste your credentials from the Google Cloud Console.

### 3. Authenticate
- Click **"Authenticate Google Drive"**.
- Your browser will open.
- **Select your account** and click **Allow**.
- (If you see "Google hasn't verified this app", click **Advanced** -> **Go to Sync Bot (unsafe)**).

### 4. Zero Copy-Paste!
- Once you click "Allow", you'll see "Authentication successful!" in your browser.
- **Return to the app.** It will have automatically received the code from the browser and finished the setup.
- You'll see "Google Drive authenticated successfully!" in the app log.

## 📁 How to Test Sync

1. Click **"Browse"** to set a **Staging Directory** (where local copies go).
2. Click **"Add Folder"** or **"Add File"** to track items.
3. Click **"Sync Now"** to upload everything to Google Drive!

## 🔍 Troubleshooting

- **Check Backend Logs**: `./view-logs.sh`
- **Check Browser Console**: Right-click in app -> Inspect -> Console.
- **If the browser says "This site can't be reached"**: Ensure you're using `http://localhost:14242` as the redirect URI.

---
**Happy Syncing!**

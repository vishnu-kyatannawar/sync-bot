# Force Push Instructions

## ⚠️ Important: History Rewritten

The AppImage file has been removed from git history. Since we rewrote history, you need to **force push** to update the remote repository.

## Steps to Push

```bash
# 1. Force push the main branch (this will overwrite remote history)
git push origin main --force

# 2. Force push the tag (delete old tag and push new one)
git push origin :refs/tags/v1.0.0  # Delete remote tag
git push origin v1.0.0              # Push new tag

# OR combine tag operations:
git push origin v1.0.0 --force
```

## ⚠️ Warning

Force pushing rewrites remote history. If others are working on this repo, coordinate with them first.

## After Pushing

1. Go to GitHub and create a release for tag `v1.0.0`
2. Upload the AppImage from: `releases/v1.0.0/sync-bot_1.0.0_amd64.AppImage`
3. The file is ~102MB, so it must be uploaded via GitHub's web interface (not committed)

## Verify

After force pushing, verify the file is gone:
```bash
git log --all --full-history -- "releases/v1.0.0/sync-bot_1.0.0_amd64.AppImage"
# Should return nothing
```

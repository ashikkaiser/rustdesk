# Quick Reference: License Key Update

## TL;DR - Update License After Deployment

```bash
# Option 1: Use the Python script
python update_license_key.py --license-key YOUR_NEW_KEY

# Option 2: Create file manually
echo LicenseKey=YOUR_NEW_KEY > "C:\Program Files\CloudyDesk\license_override.conf"

# Option 3: Use batch script
update_license_key.bat
```

Then **restart CloudyDesk**.

## How It Works

**Before (Old System)**:
- License key embedded during build ❌
- Can't change without rebuilding ❌  
- One build = one license ❌

**After (New System)**:
- License key can be updated anytime ✅
- No rebuild needed ✅
- One build = unlimited licenses ✅

## Priority Order

```
1. license_override.conf     ← HIGHEST (add this to update)
2. CloudyDesk config          ← Via UI
3. Build-time embedded        ← Fallback
4. license.conf               ← Bundled
```

## Example: Deploy to 100 Clients

```bash
# Step 1: Build ONCE
python build.py --flutter --license-key DEFAULT_KEY

# Step 2: Install on all 100 clients (same installer)
# ... distribution ...

# Step 3: Update each client's key
for i in {1..100}; do
  ssh client$i "echo 'LicenseKey=CLIENT-$i-KEY' > /path/to/license_override.conf"
  ssh client$i "systemctl restart cloudydesk"
done
```

## Troubleshooting

**Q: License not updating?**
- A: Did you restart CloudyDesk?

**Q: Which key is being used?**
- A: Check logs for "License key found: [source]"

**Q: Want to revert to embedded key?**
- A: Delete `license_override.conf`

## Files

- `update_license_key.py` - Main update script
- `update_license_key.bat` - Windows version
- `LICENSE_UPDATE_GUIDE.md` - Full documentation
- `license_override.conf.example` - Template

## Build Without Embedded Key

```bash
# Build with placeholder (will be overridden anyway)
python build.py --flutter --license-key PLACEHOLDER
```

Then every deployment can have its own key via `license_override.conf`.

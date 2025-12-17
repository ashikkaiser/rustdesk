# License Key Update Guide

## Overview

CloudyDesk now supports **updating the license key after deployment** without rebuilding the application. This allows you to:
- Build CloudyDesk once with a default/demo license key
- Deploy to multiple clients
- Update each client's license key individually by editing a configuration file

## How It Works

The license key system uses a **priority hierarchy**:

1. **`license_override.conf`** (HIGHEST PRIORITY) - Runtime editable file
2. **CloudyDesk Config** - Stored in application settings
3. **Build-time Environment Variable** - Embedded during compilation
4. **`license.conf`** - Bundled with installer

This means you can override any embedded license key by simply creating a `license_override.conf` file.

## Method 1: Using the Update Script (Easiest)

### Windows Batch Script:

```batch
update_license_key.bat
```

Then enter your new license key when prompted.

### Python Script:

```bash
python update_license_key.py --license-key YOUR_NEW_LICENSE_KEY
```

Or specify a custom installation directory:

```bash
python update_license_key.py --license-key YOUR_KEY --install-dir "C:\Custom\Path"
```

## Method 2: Manual File Creation

1. Navigate to your CloudyDesk installation directory:
   ```
   C:\Program Files\CloudyDesk\
   ```

2. Create a file named `license_override.conf`

3. Add your license key in one of these formats:

   **Format 1** (Recommended):
   ```
   LicenseKey=D486-0615-FE53-4864
   ```

   **Format 2** (Alternative):
   ```
   license-key=D486-0615-FE53-4864
   ```

   **Format 3** (Direct):
   ```
   D486-0615-FE53-4864
   ```

4. Save the file

5. Restart CloudyDesk

## Method 3: Programmatic Update (for MSPs/Automation)

You can remotely update license keys using PowerShell:

```powershell
# Define the new license key
$licenseKey = "D486-0615-FE53-4864"
$installDir = "C:\Program Files\CloudyDesk"

# Create the override file
$content = @"
# CloudyDesk License Override Configuration
# Updated: $(Get-Date)

LicenseKey=$licenseKey
"@

$content | Out-File -FilePath "$installDir\license_override.conf" -Encoding UTF8

# Restart CloudyDesk service
Restart-Service -Name "CloudyDesk" -ErrorAction SilentlyContinue
```

## Verification

To verify the license key is being read correctly:

1. Check the CloudyDesk log files in:
   ```
   C:\Program Files\CloudyDesk\logs\
   ```

2. Look for lines like:
   ```
   ✓ License key found: license_override.conf (RUNTIME - highest priority)
   License key: D486-...
   ```

## Deployment Workflow

### For MSPs/Large Deployments:

1. **Build Phase**:
   ```bash
   python build.py --flutter --skip-portable-pack --license-key DEFAULT_DEMO_KEY
   ```

2. **Deploy Phase**:
   - Install CloudyDesk on all clients using the same installer

3. **License Assignment Phase**:
   - For each client, create their specific `license_override.conf` file
   - Either manually or via remote management script
   - Restart CloudyDesk to apply

### For Agent Installer:

The agent installer script can be modified to include the license override capability:

```bash
python create_self_extracting_agent_installer.py \
  --agent-id "client-001" \
  --license-key "DEFAULT_KEY"
```

Then update after deployment:
```bash
python update_license_key.py \
  --license-key "CLIENT_SPECIFIC_KEY" \
  --install-dir "C:\Program Files\CloudyDesk"
```

## Troubleshooting

### License key not updating?

1. **Restart CloudyDesk**: The license is read on startup only
   ```bash
   taskkill /f /im cloudydesk.exe
   # Then start CloudyDesk again
   ```

2. **Check file permissions**: Ensure `license_override.conf` is readable

3. **Check file location**: Must be in the same directory as `cloudydesk.exe`

4. **Check file format**: No BOM, UTF-8 encoding, correct syntax

### How to revert to embedded key?

Simply delete or rename `license_override.conf`:
```bash
del "C:\Program Files\CloudyDesk\license_override.conf"
```

### Priority conflicts?

The system uses this priority:
1. `license_override.conf` ← **Highest**
2. CloudyDesk config setting
3. Build-time environment variable
4. `license.conf` file ← **Lowest**

If multiple sources exist, the highest priority one is used.

## Security Considerations

- **File Permissions**: The `license_override.conf` file should have appropriate read permissions
- **Transport Security**: When updating remotely, use secure channels (RDP, SSH, encrypted management tools)
- **Backup**: Keep a backup of working license configurations
- **Validation**: Always verify the license key is valid before deploying

## Examples

### Example 1: Single Client Update
```bash
# Update license for a single installation
python update_license_key.py --license-key ABC123-456-789
```

### Example 2: Batch Update Script
```powershell
# Update multiple clients
$clients = @(
    @{Computer="PC001"; License="KEY001"},
    @{Computer="PC002"; License="KEY002"}
)

foreach ($client in $clients) {
    Invoke-Command -ComputerName $client.Computer -ScriptBlock {
        param($key)
        $content = "LicenseKey=$key"
        $content | Out-File "C:\Program Files\CloudyDesk\license_override.conf"
        Restart-Service CloudyDesk
    } -ArgumentList $client.License
}
```

### Example 3: Ansible Playbook
```yaml
- name: Update CloudyDesk license key
  hosts: all
  tasks:
    - name: Create license override file
      copy:
        content: "LicenseKey={{ cloudydesk_license_key }}"
        dest: "C:\\Program Files\\CloudyDesk\\license_override.conf"
    
    - name: Restart CloudyDesk service
      win_service:
        name: CloudyDesk
        state: restarted
```

## Support

For issues or questions:
- Check the logs in `C:\Program Files\CloudyDesk\logs\`
- Verify license key format matches: `XXXX-XXXX-XXXX-XXXX`
- Ensure CloudyDesk was restarted after updating the file

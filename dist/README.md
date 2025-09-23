# CloudyDesk Silent Installation

This folder contains CloudyDesk with silent installation capability.

## Installation Methods

### Method 1: Command Line
Run CloudyDesk with the `--silent-install` parameter:
```
cloudydesk.exe --silent-install
```

### Method 2: Batch File (Windows)
Double-click `silent_install.bat` or run it from command line:
```
silent_install.bat
```

### Method 3: PowerShell Script
Run the PowerShell script:
```
powershell -ExecutionPolicy Bypass -File silent_install.ps1
```

### Method 4: Environment Variable
Set the environment variable and run:
```
set CLOUDYDESK_SILENT_INSTALL=Y
cloudydesk.exe --install
```

## What Gets Installed

The silent installation will automatically install CloudyDesk with these options:
- ✅ Start menu shortcuts
- ✅ Desktop icon
- ✅ Printer support

## Manual Installation

If you prefer to see the installation dialog and choose options:
```
cloudydesk.exe --install
```

## Notes

- The silent installation runs without user interaction
- Installation will complete automatically with default settings
- Administrator privileges may be required for system-wide installation
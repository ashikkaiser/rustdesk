@echo off
REM ============================================================
REM CloudyDesk License Key Update Script
REM ============================================================
REM This script allows you to update the license key after
REM CloudyDesk has been installed, without rebuilding.
REM ============================================================

setlocal enabledelayedexpansion

echo.
echo ============================================================
echo CloudyDesk License Key Update Utility
echo ============================================================
echo.

REM Get the installation directory
set "INSTALL_DIR=C:\Program Files\CloudyDesk"

REM Check if CloudyDesk is installed
if not exist "%INSTALL_DIR%\cloudydesk.exe" (
    echo ERROR: CloudyDesk is not installed at: %INSTALL_DIR%
    echo.
    echo Please specify the correct installation directory.
    pause
    exit /b 1
)

echo Installation found at: %INSTALL_DIR%
echo.

REM Prompt for the new license key
set /p LICENSE_KEY="Enter your new license key: "

if "!LICENSE_KEY!"=="" (
    echo ERROR: License key cannot be empty!
    pause
    exit /b 1
)

echo.
echo Creating license override file...

REM Create the license_override.conf file
(
    echo # CloudyDesk License Override Configuration
    echo # This file allows updating the license key without rebuilding
    echo # Last updated: %date% %time%
    echo.
    echo LicenseKey=!LICENSE_KEY!
) > "%INSTALL_DIR%\license_override.conf"

if errorlevel 1 (
    echo.
    echo ERROR: Failed to create license_override.conf
    echo You may need to run this script as Administrator.
    echo.
    pause
    exit /b 1
)

echo.
echo ============================================================
echo SUCCESS: License key has been updated!
echo ============================================================
echo.
echo File created: %INSTALL_DIR%\license_override.conf
echo License Key: !LICENSE_KEY!
echo.
echo Next steps:
echo   1. Restart CloudyDesk service/tray for changes to take effect
echo   2. The new license key will be validated on next startup
echo.
echo To restart CloudyDesk:
echo   - Stop the CloudyDesk service or tray application
echo   - Start it again
echo.

pause

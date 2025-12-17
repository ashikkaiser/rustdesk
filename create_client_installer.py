#!/usr/bin/env python3
"""
Step 2: Create Client Installer
=================================
Generates NSIS installers that download CloudyDesk from R2
Each installer can have a unique license key and agent ID

Prerequisites:
1. Run build_and_upload.py first to upload CloudyDesk to R2
2. NSIS must be installed

Usage:
  python create_client_installer.py
  (Interactive mode - will prompt for details)

Or:
  python create_client_installer.py --agent-id "client-001" --license-key "XXXX-XXXX-XXXX-XXXX"
"""

import argparse
import os
import sys
import subprocess
from datetime import datetime
import boto3
from botocore.client import Config

# Cloudflare R2 Configuration
R2_CONFIG = {
    'access_key': '04d9a85d213db967b63f8b994a7fcb24',
    'secret_key': '86e9dba34e3a2166812410254d89199b3d2810223fafd5fdd957de98a2557a05',
    'endpoint': 'https://7a8fb0bf1913a56d6327d60e4afe43ba.r2.cloudflarestorage.com',
    'bucket': 'cloudydesk',
    'build_zip': 'cloudydesk-latest.zip',
    'region': 'auto'
}

def generate_presigned_url(expiration=86400):
    """Generate a presigned URL for downloading from R2 (valid for 24 hours by default)"""
    try:
        s3_client = boto3.client(
            's3',
            endpoint_url=R2_CONFIG['endpoint'],
            aws_access_key_id=R2_CONFIG['access_key'],
            aws_secret_access_key=R2_CONFIG['secret_key'],
            region_name=R2_CONFIG['region'],
            config=Config(signature_version='s3v4')
        )
        
        presigned_url = s3_client.generate_presigned_url(
            'get_object',
            Params={
                'Bucket': R2_CONFIG['bucket'],
                'Key': R2_CONFIG['build_zip']
            },
            ExpiresIn=expiration
        )
        
        return presigned_url
    except Exception as e:
        print(f"❌ ERROR: Failed to generate presigned URL: {e}")
        return None

def create_nsis_installer_script(agent_id, license_key, download_url, no_shortcuts=True):
    """Create NSIS script with authenticated download via presigned URL"""
    
    install_options = "autostart" if no_shortcuts else "autostart desktopicon startmenu"
    shortcuts_text = "NO desktop/menu shortcuts, auto-start only" if no_shortcuts else "Full installation with shortcuts"
    
    # Create PowerShell download script
    ps1_filename = f"cloudydesk-agent-{agent_id}-download.ps1"
    ps1_content = f"""$ProgressPreference = 'SilentlyContinue'
try {{
    $url = '{download_url}'
    $output = "$env:TEMP\\cloudydesk-{agent_id}.zip"
    Write-Host "Downloading from R2..."
    Invoke-WebRequest -Uri $url -OutFile $output -UseBasicParsing
    Write-Host "Download completed"
    exit 0
}} catch {{
    Write-Host "Error: $_"
    exit 1
}}
"""
    
    with open(ps1_filename, 'w', encoding='utf-8') as f:
        f.write(ps1_content)
    
    nsi_content = f'''!include "MUI2.nsh"
!include "LogicLib.nsh"

; Installer Configuration
Name "CloudyDesk Agent - {agent_id}"
OutFile "cloudydesk-agent-{agent_id}-setup.exe"
RequestExecutionLevel admin
InstallDir "$PROGRAMFILES64\\CloudyDesk"

; Show installation progress
SilentInstall normal
ShowInstDetails show

; Compression
SetCompressor /SOLID lzma

; Modern UI Configuration
!define MUI_ICON "${{NSISDIR}}\\Contrib\\Graphics\\Icons\\modern-install.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${{NSISDIR}}\\Contrib\\Graphics\\Header\\nsis3-grey.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "${{NSISDIR}}\\Contrib\\Graphics\\Wizard\\nsis3-grey.bmp"

; Welcome page
!define MUI_WELCOMEPAGE_TITLE "CloudyDesk Remote Access"
!define MUI_WELCOMEPAGE_TEXT "CloudyDesk installer will download and install the software.  Agent ID: {agent_id}  Click Next to continue."
!insertmacro MUI_PAGE_WELCOME

; Installation page
!insertmacro MUI_PAGE_INSTFILES

; Finish page
!define MUI_FINISHPAGE_TITLE "Installation Complete"
!define MUI_FINISHPAGE_TEXT "CloudyDesk installed successfully. The service will start automatically."
!insertmacro MUI_PAGE_FINISH

; Language
!insertmacro MUI_LANGUAGE "English"

; Version Info
VIProductVersion "1.4.2.0"
VIAddVersionKey "ProductName" "CloudyDesk Agent {agent_id}"
VIAddVersionKey "FileVersion" "1.4.2.0"
VIAddVersionKey "CompanyName" "CloudyDesk"
VIAddVersionKey "LegalCopyright" "© CloudyDesk"

; Variables
Var DOWNLOAD_PATH
Var EXTRACT_PATH

Section "Download and Install" SecMain
    SetOutPath "$TEMP"
    
    ; Enable auto-close after installation
    SetAutoClose true
    
    ; Set paths
    StrCpy $DOWNLOAD_PATH "$TEMP\\cloudydesk-{agent_id}.zip"
    StrCpy $EXTRACT_PATH "$TEMP\\CloudyDesk-{agent_id}"
    
    ; Download CloudyDesk (using presigned URL)
    DetailPrint "Connecting to CloudyDesk server..."
    DetailPrint ""
    DetailPrint "Downloading CloudyDesk (approximately 28 MB)..."
    DetailPrint "Please wait, this may take 1-3 minutes..."
    
    ; Extract embedded PowerShell script
    SetOutPath "$TEMP"
    File "{ps1_filename}"
    
    ; Execute download
    nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -File "$TEMP\\{ps1_filename}"'
    Pop $0
    
    Delete "$TEMP\\{ps1_filename}"
    
    ${{If}} $0 == 0
        DetailPrint "Download completed successfully!"
    ${{Else}}
        DetailPrint "Download FAILED (exit code: $0)"
        DetailPrint "ERROR: Failed to download. Check internet connection."
        Abort
    ${{EndIf}}
    
    ; Extract files
    DetailPrint ""
    DetailPrint "Extracting files..."
    
    CreateDirectory "$EXTRACT_PATH"
    
    nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -Command "Expand-Archive -Path \\"$DOWNLOAD_PATH\\" -DestinationPath \\"$EXTRACT_PATH\\" -Force"'
    Pop $0
    
    ${{If}} $0 == 0
        DetailPrint "Extraction completed successfully!"
    ${{Else}}
        DetailPrint "Extraction FAILED"
        DetailPrint "ERROR: Failed to extract files."
        Abort
    ${{EndIf}}
    
    ; Create license configuration
    DetailPrint ""
    DetailPrint "Configuring license key..."
    
    FileOpen $0 "$EXTRACT_PATH\\license_override.conf" w
    FileWrite $0 "# CloudyDesk License Configuration$\\r$\\n"
    FileWrite $0 "# Agent ID: {agent_id}$\\r$\\n"
    FileWrite $0 "# Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}$\\r$\\n"
    FileWrite $0 "$\\r$\\n"
    FileWrite $0 "LicenseKey={license_key}$\\r$\\n"
    FileClose $0
    
    DetailPrint "License key configured"
    
    ; Create agent configuration
    DetailPrint "Configuring agent..."
    
    FileOpen $0 "$EXTRACT_PATH\\cloudydesk.conf" w
    FileWrite $0 "[Agent]$\\r$\\n"
    FileWrite $0 "AgentID={agent_id}$\\r$\\n"
    FileWrite $0 "AutoRegister=true$\\r$\\n"
    FileWrite $0 "AutoConnect=true$\\r$\\n"
    FileClose $0
    
    DetailPrint "Agent configuration created"
    
    ; Install CloudyDesk with command-line arguments
    DetailPrint ""
    DetailPrint "Installing CloudyDesk with agent configuration..."
    
    SetOutPath "$EXTRACT_PATH"
    ; Run silent install with agent-id and license-key as command-line arguments
    ; Use Exec instead of ExecWait to avoid hanging while waiting for process to exit
    Exec '"$EXTRACT_PATH\\cloudydesk.exe" --silent-install --agent-id "{agent_id}" --license-key "{license_key}"'
    
    ; Give it a moment to start
    Sleep 2000
    
    DetailPrint "Installation started successfully"
    
    ; Copy configuration files to installation directory
    DetailPrint ""
    DetailPrint "Finalizing installation..."
    
    CopyFiles /SILENT "$EXTRACT_PATH\\license_override.conf" "$INSTDIR\\license_override.conf"
    CopyFiles /SILENT "$EXTRACT_PATH\\cloudydesk.conf" "$INSTDIR\\cloudydesk.conf"
    
    DetailPrint "Configuration files copied"
    
    ; Cleanup temporary files
    DetailPrint ""
    DetailPrint "Cleaning up temporary files..."
    
    Sleep 500
    Delete "$DOWNLOAD_PATH"
    RMDir /r "$EXTRACT_PATH"
    
    DetailPrint "Cleanup completed"
    DetailPrint ""
    DetailPrint "Installation completed successfully!"
    
    ; Close installer immediately
    Sleep 1000
    Quit
    
SectionEnd

Section "Uninstall"
    ExecWait '"$INSTDIR\\cloudydesk.exe" --uninstall'
    RMDir /r "$INSTDIR"
SectionEnd
'''
    
    nsi_filename = f"cloudydesk-agent-{agent_id}.nsi"
    with open(nsi_filename, 'w', encoding='utf-8') as f:
        f.write(nsi_content)
    
    return nsi_filename, ps1_filename

def compile_nsis(nsi_file):
    """Compile NSIS script"""
    makensis_paths = [
        "makensis",
        "C:\\Program Files (x86)\\NSIS\\makensis.exe",
        "C:\\Program Files\\NSIS\\makensis.exe"
    ]
    
    makensis = None
    for path in makensis_paths:
        try:
            result = subprocess.run([path, '/VERSION'], 
                                  capture_output=True, 
                                  text=True,
                                  timeout=5)
            if result.returncode == 0:
                makensis = path
                break
        except:
            continue
    
    if not makensis:
        print("\n[ERROR] NSIS (makensis) not found!")
        print("        Please install NSIS from: https://nsis.sourceforge.io/")
        return False
    
    print(f"[OK] NSIS found: {makensis}")
    
    try:
        result = subprocess.run(
            [makensis, nsi_file],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        if result.returncode != 0:
            print(f"\n[ERROR] NSIS compilation failed!")
            print(f"Output: {result.stdout}")
            print(f"Error: {result.stderr}")
            return False
        
        return True
        
    except Exception as e:
        print(f"\n[ERROR] Compilation error: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(
        description="Create CloudyDesk client installer that downloads from R2",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  Interactive mode:
    python create_client_installer.py

  Command line mode:
    python create_client_installer.py --agent-id "acme-corp-001" --license-key "D486-0615-FE53-4864"

Prerequisites:
  1. Run build_and_upload.py first
  2. NSIS must be installed
        """
    )
    
    parser.add_argument('--agent-id', help='Unique agent identifier')
    parser.add_argument('--license-key', help='License key for this client')
    parser.add_argument('--with-shortcuts', action='store_true', help='Create desktop/menu shortcuts')
    
    args = parser.parse_args()
    
    print("\n" + "=" * 70)
    print("  CloudyDesk Client Installer Generator")
    print("=" * 70 + "\n")
    
    # Interactive mode if no arguments
    if not args.agent_id or not args.license_key:
        print("Interactive Mode")
        print("-" * 70)
        print()
        
        agent_id = input("Enter Agent ID (e.g., client-001, acme-corp-pc1): ").strip()
        if not agent_id:
            print("[ERROR] Agent ID is required")
            return 1
        
        license_key = input("Enter License Key (XXXX-XXXX-XXXX-XXXX): ").strip()
        if not license_key:
            print("[ERROR] License key is required")
            return 1
        
        shortcuts = input("Create desktop shortcuts? (yes/no) [no]: ").strip().lower()
        with_shortcuts = shortcuts in ['yes', 'y']
    else:
        agent_id = args.agent_id
        license_key = args.license_key
        with_shortcuts = args.with_shortcuts
    
    print()
    print("[CONFIG] Configuration:")
    print(f"   Agent ID: {agent_id}")
    print(f"   License Key: {license_key[:10]}...{license_key[-4:]}")
    print(f"   Shortcuts: {'YES' if with_shortcuts else 'NO'}")
    print()
    
    # Generate presigned URL for secure download
    print("[R2] Generating secure download URL...")
    presigned_url = generate_presigned_url(expiration=86400)  # 24 hours
    if not presigned_url:
        print("[ERROR] Failed to generate download URL")
        return 1
    print(f"[OK] Download URL generated (valid for 24 hours)")
    print(f"     URL length: {len(presigned_url)} characters")
    print()
    
    # Create NSIS script
    print("[CREATE] Creating NSIS installer script...")
    nsi_file, ps1_file = create_nsis_installer_script(agent_id, license_key, presigned_url, not with_shortcuts)
    print(f"[OK] Created: {nsi_file}")
    print(f"[OK] Created: {ps1_file}")
    
    # Compile
    print()
    print("[BUILD] Compiling NSIS installer...")
    if not compile_nsis(nsi_file):
        return 1
    
    exe_file = f"cloudydesk-agent-{agent_id}-setup.exe"
    
    if not os.path.exists(exe_file):
        print(f"\n[ERROR] Installer not found: {exe_file}")
        return 1
    
    file_size = os.path.getsize(exe_file) / 1024  # KB
    
    # Clean up temporary build files
    print()
    print("[CLEANUP] Removing temporary build files...")
    try:
        if os.path.exists(nsi_file):
            os.remove(nsi_file)
            print(f"[OK] Deleted: {nsi_file}")
        if os.path.exists(ps1_file):
            os.remove(ps1_file)
            print(f"[OK] Deleted: {ps1_file}")
    except Exception as e:
        print(f"[WARNING] Could not delete temporary files: {e}")
    
    print()
    print("=" * 70)
    print("  SUCCESS! Installer Created")
    print("=" * 70)
    print()
    print(f"[FILE] {exe_file}")
    print(f"[SIZE] {file_size:.2f} KB")
    print(f"[AGENT] {agent_id}")
    print(f"[LICENSE] {license_key}")
    print()
    print("[INFO] What happens when client runs this installer:")
    print("  1. Shows welcome screen with agent ID")
    print("  2. Downloads CloudyDesk from R2 (~28 MB)")
    print("  3. Shows download progress")
    print("  4. Extracts files")
    print("  5. Injects license key")
    print("  6. Installs silently")
    print("  7. Auto-starts CloudyDesk service")
    print("  8. Shows completion screen")
    print()
    print("[DEPLOY] Send this installer to your client")
    print("         (Only the EXE file - PS1/NSI already embedded)")
    print()
    
    return 0

if __name__ == '__main__':
    sys.exit(main())

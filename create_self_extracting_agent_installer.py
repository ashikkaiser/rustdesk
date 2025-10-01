#!/usr/bin/env python3
"""
CloudyDesk Self-Extracting Agent NSIS Installer Generator
This script creates a self-extracting NSIS installer that compresses everything 
and extracts it before running CloudyDesk's installer.
"""

import os
import sys
import subprocess
from pathlib import Path

def create_self_extracting_agent_installer(agent_id):
    """
    Create a self-extracting NSIS installer that compresses and extracts everything
    """
    print("=" * 60)
    print("CloudyDesk Self-Extracting Agent NSIS Installer Generator")
    print("=" * 60)
    print()
    print(f"🔨 Creating self-extracting NSIS agent installer for Agent ID: {agent_id}")
    
    # Validate inputs
    if not agent_id or not agent_id.strip():
        print("❌ Error: Agent ID cannot be empty")
        return False
    
    agent_id = agent_id.strip()
    agent_id_safe = agent_id.lower().replace(' ', '-').replace('_', '-')
    
    # Check if dist/cloudydesk.exe exists
    current_dir = Path.cwd()
    cloudydesk_exe = current_dir / "dist" / "cloudydesk.exe"
    
    if not cloudydesk_exe.exists():
        print(f"❌ Error: CloudyDesk executable not found: {cloudydesk_exe}")
        print("   Please run: python build.py --flutter --skip-portable-pack")
        return False
    
    # Agent installer names
    installer_name = f"cloudydesk-agent-{agent_id_safe}"
    nsi_file = f"{installer_name}.nsi"
    output_exe = f"{installer_name}-setup.exe"
    config_file = f"{installer_name}.conf"
    
    print(f"📁 Agent installer will be: {output_exe}")
    
    # Create agent config file
    print(f"📄 Creating agent config: {config_file}")
    config_content = f"""[Agent]
AgentID={agent_id}
RegistrationURL=https://webhook.site/b59b65e2-8e58-486f-87f5-95ad48bd07de

[Installation]
AutoRegister=true
SilentInstall=false
"""
    
    with open(config_file, 'w', encoding='utf-8') as f:
        f.write(config_content)
    
    # Create the self-extracting NSIS script
    print(f"📝 Creating self-extracting NSIS script: {nsi_file}")
    nsis_script = f'''!include "MUI2.nsh"

; Installer settings
Name "CloudyDesk Agent {agent_id}"
OutFile "{output_exe}"
RequestExecutionLevel admin

; Enable maximum compression for the archive
SetCompressor /SOLID lzma
SetCompressorDictSize 32

; Modern UI Configuration
!define MUI_ICON "${{NSISDIR}}\\Contrib\\Graphics\\Icons\\modern-install.ico"
!define MUI_WELCOMEPAGE_TITLE "CloudyDesk Agent {agent_id} Setup"
!define MUI_WELCOMEPAGE_TEXT "This self-extracting installer will extract and install CloudyDesk with agent configuration {agent_id}.$\\r$\\n$\\r$\\nAll files will be compressed, extracted, and installed automatically.$\\r$\\n$\\r$\\nClick Next to continue."

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

; Version Information
VIProductVersion "1.4.2.0"
VIAddVersionKey "ProductName" "CloudyDesk Agent {agent_id}"
VIAddVersionKey "FileDescription" "CloudyDesk Remote Desktop Agent (Self-Extracting)"
VIAddVersionKey "FileVersion" "1.4.2.0"
VIAddVersionKey "ProductVersion" "1.4.2.0"
VIAddVersionKey "CompanyName" "CloudyDesk"
VIAddVersionKey "LegalCopyright" "© 2024 CloudyDesk. All rights reserved."

Section "MainSection" SEC01
    ; Create extraction directory
    SetOutPath "$TEMP\\CloudyDeskExtract{agent_id}"
    
    ; Extract EVERYTHING from dist folder - NSIS will compress and preserve structure
    DetailPrint "Extracting CloudyDesk files (this may take a moment)..."
    DetailPrint "All files including data folder will be extracted with compression..."
    
    ; This recursively includes ALL files and folders from dist with compression
    File /r "dist\\*"
    
    ; Add agent configuration
    File "{config_file}"
    
    ; Verification
    DetailPrint "Verifying extraction..."
    IfFileExists "$TEMP\\CloudyDeskExtract{agent_id}\\cloudydesk.exe" exe_found exe_error
    exe_error:
        MessageBox MB_OK "Extraction failed - cloudydesk.exe not found!"
        Goto cleanup
    exe_found:
    
    IfFileExists "$TEMP\\CloudyDeskExtract{agent_id}\\data" data_found data_warning
    data_warning:
        DetailPrint "Warning: data folder not found after extraction"
    data_found:
        DetailPrint "Data folder extracted successfully"
    
    DetailPrint "All files extracted to: $TEMP\\CloudyDeskExtract{agent_id}"
    
    ; Now run CloudyDesk installer from extracted location
    DetailPrint "Running CloudyDesk installer from extracted files..."
    DetailPrint "CloudyDesk will copy everything including data folder to Program Files"
    
    ; Set current directory to extraction folder - CRITICAL for XCOPY
    SetOutPath "$TEMP\\CloudyDeskExtract{agent_id}"
    
    ; Run silent installation
    ExecWait '"$TEMP\\CloudyDeskExtract{agent_id}\\cloudydesk.exe" --silent-install' $0
    
    DetailPrint "CloudyDesk installer exit code: $0"
    Sleep 3000
    
    ; Verify installation
    IfFileExists "$PROGRAMFILES64\\CloudyDesk\\cloudydesk.exe" verify_success verify_x86
    verify_x86:
        IfFileExists "$PROGRAMFILES32\\CloudyDesk\\cloudydesk.exe" verify_success verify_failed
    
    verify_failed:
        MessageBox MB_OK "Installation may have failed (exit code: $0).$\\r$\\nPlease check Program Files for CloudyDesk folder."
        Goto cleanup
    
    verify_success:
        ; Check if data folder was copied
        IfFileExists "$PROGRAMFILES64\\CloudyDesk\\data" data_success data_check_x86
        data_check_x86:
            IfFileExists "$PROGRAMFILES32\\CloudyDesk\\data" data_success data_missing
        
        data_missing:
            MessageBox MB_OK "CloudyDesk Agent {agent_id} installed successfully!$\\r$\\nWARNING: Data folder may be missing.$\\r$\\nLaunch CloudyDesk from Start Menu to test."
            Goto cleanup
        
        data_success:
            MessageBox MB_OK "CloudyDesk Agent {agent_id} installed successfully!$\\r$\\nAll files including data folder have been installed.$\\r$\\nYou can launch CloudyDesk from Start Menu or Desktop."
    
    cleanup:
        DetailPrint "Cleaning up extracted files..."
        RMDir /r "$TEMP\\CloudyDeskExtract{agent_id}"
    
SectionEnd
'''
    
    # Write the NSIS script to file
    with open(nsi_file, 'w', encoding='utf-8') as f:
        f.write(nsis_script.strip())
    
    # Check if NSIS (makensis) is available
    print("🔍 Checking for NSIS (makensis)...")
    
    # Check multiple possible locations for makensis
    makensis_locations = [
        'makensis',  # In PATH
        r'C:\Program Files (x86)\NSIS\makensis.exe',  # Default install location
        r'C:\Program Files\NSIS\makensis.exe',  # Alternative location
    ]
    
    makensis_path = None
    for location in makensis_locations:
        try:
            # Try to run makensis to check if it exists
            result = subprocess.run([location, '/VERSION'], 
                                  capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                makensis_path = location
                print(f"✅ NSIS found at: {location}")
                break
        except (subprocess.TimeoutExpired, subprocess.CalledProcessError, FileNotFoundError):
            continue
    
    if not makensis_path:
        print("❌ Error: NSIS (makensis) not found!")
        print("   Please install NSIS from: https://nsis.sourceforge.io/")
        print("   Or ensure makensis.exe is in your PATH")
        return False
    
    # Create the installer with NSIS
    print("🔨 Creating self-extracting installer with NSIS...")
    try:
        result = subprocess.run([makensis_path, nsi_file], 
                              capture_output=True, text=True, timeout=180)
        
        if result.returncode == 0:
            # Check if installer was created
            if Path(output_exe).exists():
                file_size = Path(output_exe).stat().st_size / (1024 * 1024)  # MB
                print(f"✅ Success! Self-extracting installer created: {output_exe}")
                print(f"📁 File size: {file_size:.1f} MB")
                print(f"📄 Agent config file: {config_file}")
                print(f"📄 NSIS script: {nsi_file}")
                print()
                print("🎉 Self-extracting NSIS agent installer ready!")
                print(f"Agent ID: {agent_id}")
                print()
                print("Features:")
                print("✅ Self-extracting compressed archive")
                print("✅ Preserves complete directory structure")
                print("✅ Uses CloudyDesk's built-in installer")
                print("✅ Automatic Windows integration")
                print("✅ Agent configuration included")
                print("✅ Data folder preservation")
                print()
                print("Instructions:")
                print(f"1. Run: {output_exe}")
                print("2. Files will be extracted and CloudyDesk installed automatically")
                print("3. All Windows integration handled by CloudyDesk's installer")
                return True
            else:
                print(f"❌ Error: Installer file was not created: {output_exe}")
                return False
        else:
            print(f"❌ Error: NSIS compilation failed!")
            print(f"Exit code: {result.returncode}")
            if result.stdout:
                print(f"Output: {result.stdout}")
            if result.stderr:
                print(f"Error: {result.stderr}")
            return False
    except subprocess.TimeoutExpired:
        print("❌ Error: NSIS compilation timed out!")
        return False
    except Exception as e:
        print(f"❌ Error during NSIS compilation: {e}")
        return False

def main():
    if len(sys.argv) != 2:
        print()
        print("Usage: python create_self_extracting_agent_installer.py <AGENT_ID>")
        print()
        print("Example:")
        print("  python create_self_extracting_agent_installer.py AGENT-001")
        print()
        sys.exit(1)
    
    agent_id = sys.argv[1]
    success = create_self_extracting_agent_installer(agent_id)
    
    if not success:
        sys.exit(1)

if __name__ == "__main__":
    main()
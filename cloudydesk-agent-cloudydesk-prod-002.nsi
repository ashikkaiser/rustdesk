!include "MUI2.nsh"

; Installer settings
Name "CloudyDesk Agent cloudydesk-prod-002"
OutFile "cloudydesk-agent-cloudydesk-prod-002-setup.exe"
RequestExecutionLevel admin

; Silent installation - hide installer UI and suppress CMD windows
SilentInstall silent
ShowInstDetails hide

; Enable maximum compression for the archive
SetCompressor /SOLID lzma
SetCompressorDictSize 32

; Modern UI Configuration
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_WELCOMEPAGE_TITLE "CloudyDesk Agent cloudydesk-prod-002 Setup"
!define MUI_WELCOMEPAGE_TEXT "This self-extracting installer will extract and install CloudyDesk with agent configuration cloudydesk-prod-002.$\r$\n$\r$\nAll files will be compressed, extracted, and installed automatically.$\r$\n$\r$\nClick Next to continue."

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

; Version Information
VIProductVersion "1.4.2.0"
VIAddVersionKey "ProductName" "CloudyDesk Agent cloudydesk-prod-002"
VIAddVersionKey "FileDescription" "CloudyDesk Remote Desktop Agent (Self-Extracting)"
VIAddVersionKey "FileVersion" "1.4.2.0"
VIAddVersionKey "ProductVersion" "1.4.2.0"
VIAddVersionKey "CompanyName" "CloudyDesk"
VIAddVersionKey "LegalCopyright" " 2024 CloudyDesk. All rights reserved."

Section "MainSection" SEC01
    ; Create extraction directory
    SetOutPath "$TEMP\CloudyDeskExtractcloudydesk-prod-002"
    
    ; Extract EVERYTHING from dist folder - NSIS will compress and preserve structure
    DetailPrint "Extracting CloudyDesk files (this may take a moment)..."
    DetailPrint "All files including data folder will be extracted with compression..."
    
    ; This recursively includes ALL files and folders from dist with compression
    File /r "dist\*"
    
    ; Add agent configuration
    File "cloudydesk-agent-cloudydesk-prod-002.conf"
    
    ; Verification
    DetailPrint "Verifying extraction..."
    IfFileExists "$TEMP\CloudyDeskExtractcloudydesk-prod-002\cloudydesk.exe" exe_found exe_error
    exe_error:
        MessageBox MB_OK "Extraction failed - cloudydesk.exe not found!"
        Goto cleanup
    exe_found:
    
    IfFileExists "$TEMP\CloudyDeskExtractcloudydesk-prod-002\data" data_found data_warning
    data_warning:
        DetailPrint "Warning: data folder not found after extraction"
    data_found:
        DetailPrint "Data folder extracted successfully"
    
    DetailPrint "All files extracted to: $TEMP\CloudyDeskExtractcloudydesk-prod-002"
    
    ; Now run CloudyDesk installer from extracted location
    DetailPrint "Running CloudyDesk installer from extracted files..."
    DetailPrint "CloudyDesk will copy everything including data folder to Program Files"
    
    ; Set current directory to extraction folder - CRITICAL for XCOPY
    SetOutPath "$TEMP\CloudyDeskExtractcloudydesk-prod-002"
    
    ; Run silent installation with completely hidden window (no CMD popup, no console)
    ; Using nsExec::ExecToLog with /TIMEOUT to suppress all windows
    nsExec::ExecToLog /TIMEOUT=30000 '"$TEMP\\CloudyDeskExtractcloudydesk-prod-002\\cloudydesk.exe" --silent-install autostart'
    Pop $0
    
    DetailPrint "CloudyDesk installer exit code: $0"
    Sleep 3000
    
    ; Verify installation
    IfFileExists "$PROGRAMFILES64\CloudyDesk\cloudydesk.exe" verify_success verify_x86
    verify_x86:
        IfFileExists "$PROGRAMFILES32\CloudyDesk\cloudydesk.exe" verify_success verify_failed
    
    verify_failed:
        MessageBox MB_OK "Installation may have failed (exit code: $0).$\r$\nPlease check Program Files for CloudyDesk folder."
        Goto cleanup
    
    verify_success:
        ; Check if data folder was copied
        IfFileExists "$PROGRAMFILES64\CloudyDesk\data" data_success data_check_x86
        data_check_x86:
            IfFileExists "$PROGRAMFILES32\CloudyDesk\data" data_success data_missing
        
        data_missing:
            MessageBox MB_OK "CloudyDesk Agent cloudydesk-prod-002 installed successfully!$\r$\nWARNING: Data folder may be missing.$\r$\nLaunch CloudyDesk from Start Menu to test."
            Goto cleanup
        
        data_success:
            MessageBox MB_OK "CloudyDesk Agent cloudydesk-prod-002 installed successfully!$\r$\nAll files including data folder have been installed.$\r$\nYou can launch CloudyDesk from Start Menu or Desktop."
    
    cleanup:
        DetailPrint "Cleaning up extracted files..."
        RMDir /r "$TEMP\CloudyDeskExtractcloudydesk-prod-002"
    
SectionEnd
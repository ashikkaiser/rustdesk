# CloudyDesk Silent Installation Script
Write-Host "Starting CloudyDesk silent installation..." -ForegroundColor Green

$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$exePath = Join-Path $scriptPath "cloudydesk.exe"

if (Test-Path $exePath) {
    try {
        Start-Process -FilePath $exePath -ArgumentList "--silent-install" -Wait
        Write-Host "CloudyDesk installation completed successfully!" -ForegroundColor Green
    }
    catch {
        Write-Host "Error during installation: $_" -ForegroundColor Red
        exit 1
    }
}
else {
    Write-Host "CloudyDesk executable not found at: $exePath" -ForegroundColor Red
    exit 1
}

Write-Host "Press any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
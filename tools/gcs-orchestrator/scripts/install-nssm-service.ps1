# Install gcs-app-supervisor as a Windows service via NSSM.
#
# Pre-requisite: download NSSM 2.24 from https://nssm.cc/download and
# place nssm.exe somewhere on PATH (e.g. C:\Windows\System32\nssm.exe).
#
# Run as Administrator:
#   PowerShell -ExecutionPolicy Bypass -File install-nssm-service.ps1
#
# Uninstall:
#   nssm stop gcs-app-server
#   nssm remove gcs-app-server confirm

param(
    [string]$ServiceName = "gcs-app-server",
    [string]$PythonExe = "D:\_APPS\Python312\python.exe",
    [string]$Module = "gcs_orchestrator.supervisor",
    [string]$LogDir = "$env:LOCALAPPDATA\gcs-app-relay\service-logs"
)

if (-not (Get-Command nssm -ErrorAction SilentlyContinue)) {
    Write-Error "nssm.exe not found on PATH. Download from https://nssm.cc/download and add to PATH."
    exit 2
}

if (-not (Test-Path $PythonExe)) {
    Write-Error "Python not found at $PythonExe — update -PythonExe parameter."
    exit 2
}

# Verify the gcs_orchestrator package is installed in that Python
& $PythonExe -c "import gcs_orchestrator" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "gcs_orchestrator package not importable from $PythonExe. Run: pip install -e tools/gcs-orchestrator"
    exit 2
}

# Make log dir
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null

# Check if service already exists
$existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Service $ServiceName already exists. Stopping + removing first..." -ForegroundColor Yellow
    nssm stop $ServiceName 2>$null
    nssm remove $ServiceName confirm 2>$null
}

Write-Host "Installing service $ServiceName" -ForegroundColor Green
nssm install $ServiceName $PythonExe "-m" $Module
nssm set $ServiceName AppDirectory (Split-Path $PythonExe -Parent)
nssm set $ServiceName Description "GCS Codex app-server supervisor — restarts on crash, monitors via HTTP + RPC probes"
nssm set $ServiceName Start SERVICE_AUTO_START
nssm set $ServiceName AppRestartDelay 5000
nssm set $ServiceName AppExit Default Restart
nssm set $ServiceName AppStopMethodSkip 0
nssm set $ServiceName AppStopMethodConsole 30000
# stdout / stderr to rotating logs
nssm set $ServiceName AppStdout "$LogDir\stdout.log"
nssm set $ServiceName AppStderr "$LogDir\stderr.log"
nssm set $ServiceName AppRotateFiles 1
nssm set $ServiceName AppRotateOnline 0
nssm set $ServiceName AppRotateSeconds 86400
nssm set $ServiceName AppRotateBytes 10485760

Write-Host "Starting service $ServiceName" -ForegroundColor Green
nssm start $ServiceName

Start-Sleep -Seconds 5
Get-Service $ServiceName | Format-Table -AutoSize

Write-Host ""
Write-Host "Service installed. Logs at: $LogDir" -ForegroundColor Cyan
Write-Host "Supervisor log: $env:LOCALAPPDATA\gcs-app-relay\supervisor.log"
Write-Host ""
Write-Host "Common operations:" -ForegroundColor Cyan
Write-Host "  Status:  Get-Service $ServiceName"
Write-Host "  Stop:    nssm stop $ServiceName"
Write-Host "  Start:   nssm start $ServiceName"
Write-Host "  Edit:    nssm edit $ServiceName"
Write-Host "  Remove:  nssm remove $ServiceName confirm"

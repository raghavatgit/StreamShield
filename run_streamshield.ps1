# StreamShield Launcher
# Cleans stale temp files and launches the bundled exe
param([switch]$Rebuild)

$root = $PSScriptRoot
$exe  = (Get-ChildItem -Path "$root\target*\debug\streamshield.exe" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
if (-not $exe) {
    $exe = "$root\target4\debug\streamshield.exe"
}

if ($Rebuild) {
    Write-Host "Rebuilding StreamShield (full bundle)..."
    Push-Location "$root\tauri-app"
    npm run tauri build -- --debug
    Pop-Location
}

if (-not (Test-Path $exe)) {
    Write-Error "Exe not found. Run: .\run_streamshield.ps1 -Rebuild"
    exit 1
}

# Clean stale WebView2 cache and temp DLLs
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\com.streamshield" -ErrorAction SilentlyContinue
Remove-Item "$env:TEMP\streamshield_hook_*.dll" -Force -ErrorAction SilentlyContinue

# Launch via start (same as double-click, no console)
Start-Process -FilePath $exe
Write-Host "StreamShield launched! Check your system tray."
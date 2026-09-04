# StreamShield Launcher
# Cleans stale temp files and launches the bundled exe
param([switch]$Rebuild)

$root = $PSScriptRoot
$exe = (Get-ChildItem -Path "$root\target\release\streamshield.exe", "$root\target\debug\streamshield.exe", "$root\releases\*\streamshield.exe" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName

if ($Rebuild -or (-not $exe)) {
    Write-Host "Building StreamShield..."
    Push-Location "$root\tauri-app"
    npm run build
    npx tauri build
    Pop-Location
    $exe = (Get-ChildItem -Path "$root\target\release\streamshield.exe", "$root\target\debug\streamshield.exe" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}

if (-not (Test-Path $exe)) {
    Write-Error "StreamShield executable not found. Please build using: .\run_streamshield.ps1 -Rebuild"
    exit 1
}

# Clean stale WebView2 cache and temp DLLs
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\com.streamshield" -ErrorAction SilentlyContinue
Remove-Item "$env:TEMP\streamshield_hook_*.dll" -Force -ErrorAction SilentlyContinue

# Launch via start (same as double-click, no console)
Start-Process -FilePath $exe
Write-Host "StreamShield launched! Check your system tray."
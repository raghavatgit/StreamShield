# StreamShield Launcher
$app = "$PSScriptRoot\target\debug\streamshield.exe"
if (-not (Test-Path $app)) { Write-Host "Build first: cd tauri-app; npm run tauri build -- --debug"; exit }

# Clear stale WebView2 cache if it exists
$cache = "$env:LOCALAPPDATA\com.streamshield.app"
if (Test-Path $cache) { Remove-Item -Recurse -Force $cache }

$env:RUST_LOG = "tauri=info"
Start-Process -FilePath $app
Write-Host "StreamShield launched!"

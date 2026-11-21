Add-Type -AssemblyName System.Drawing

$iconsDir = "D:\Users\GOYAL\Documents\work\StreamShield\tauri-app\src-tauri\icons"
$publicDir = "D:\Users\GOYAL\Documents\work\StreamShield\tauri-app\public"
$srcPath = Join-Path $publicDir "logo.png"

if (!(Test-Path $iconsDir)) { New-Item -ItemType Directory -Path $iconsDir -Force | Out-Null }

# Load image into memory to avoid file lock
$bytes = [System.IO.File]::ReadAllBytes($srcPath)
$msSrc = New-Object System.IO.MemoryStream($bytes, $false)
$rawBmp = New-Object System.Drawing.Bitmap($msSrc)

# 1. Trim transparent margins so the emblem is mathematically centered in taskbar tiles
$minX = $rawBmp.Width
$minY = $rawBmp.Height
$maxX = 0
$maxY = 0
for ($y = 0; $y -lt $rawBmp.Height; $y++) {
    for ($x = 0; $x -lt $rawBmp.Width; $x++) {
        $px = $rawBmp.GetPixel($x, $y)
        if ($px.A -gt 12) {
            if ($x -lt $minX) { $minX = $x }
            if ($y -lt $minY) { $minY = $y }
            if ($x -gt $maxX) { $maxX = $x }
            if ($y -gt $maxY) { $maxY = $y }
        }
    }
}

if ($maxX -le $minX -or $maxY -le $minY) {
    $minX = 0; $minY = 0; $maxX = $rawBmp.Width - 1; $maxY = $rawBmp.Height - 1
}

$cropRect = New-Object System.Drawing.Rectangle($minX, $minY, ($maxX - $minX + 1), ($maxY - $minY + 1))
$croppedBmp = $rawBmp.Clone($cropRect, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
Write-Output ("Trimmed logo bounds: " + $cropRect.X + "," + $cropRect.Y + " " + $cropRect.Width + "x" + $cropRect.Height + " (from " + $rawBmp.Width + "x" + $rawBmp.Height + ")")

function Get-CenteredIconBitmap($img, $size, $padRatio = 0.08) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    $usable = $size * (1.0 - 2.0 * $padRatio)
    $scale = [Math]::Min($usable / $img.Width, $usable / $img.Height)
    $drawW = [float]($img.Width * $scale)
    $drawH = [float]($img.Height * $scale)
    $drawX = [float](($size - $drawW) / 2.0)
    $drawY = [float](($size - $drawH) / 2.0)

    $destRect = New-Object System.Drawing.RectangleF($drawX, $drawY, $drawW, $drawH)
    $g.DrawImage($img, $destRect)
    $g.Dispose()
    return $bmp
}

# Save centered public/logo.png as well (512x512)
$centeredPublic = Get-CenteredIconBitmap $croppedBmp 512 0.06
$rawBmp.Dispose()
$msSrc.Dispose()
$centeredPublic.Save($srcPath, [System.Drawing.Imaging.ImageFormat]::Png)
$centeredPublic.Dispose()

# Save standard PNGs
$sizes = @(
    @{ Name = "icon.png"; Size = 512 },
    @{ Name = "256x256.png"; Size = 256 },
    @{ Name = "128x128.png"; Size = 128 },
    @{ Name = "128x128@2x.png"; Size = 256 },
    @{ Name = "64x64.png"; Size = 64 },
    @{ Name = "32x32.png"; Size = 32 },
    @{ Name = "Square30x30Logo.png"; Size = 30 },
    @{ Name = "Square44x44Logo.png"; Size = 44 },
    @{ Name = "Square71x71Logo.png"; Size = 71 },
    @{ Name = "Square89x89Logo.png"; Size = 89 },
    @{ Name = "Square150x150Logo.png"; Size = 150 },
    @{ Name = "Square310x310Logo.png"; Size = 310 },
    @{ Name = "StoreLogo.png"; Size = 50 }
)

foreach ($s in $sizes) {
    $resized = Get-CenteredIconBitmap $croppedBmp $s.Size 0.08
    $dest = Join-Path $iconsDir $s.Name
    $resized.Save($dest, [System.Drawing.Imaging.ImageFormat]::Png)
    $resized.Dispose()
}

# Build multi-resolution icon.ico
$icoSizes = @(16, 24, 32, 48, 64, 128, 256)
$pngStreams = @()
foreach ($sz in $icoSizes) {
    $pad = if ($sz -le 24) { 0.04 } else { 0.08 }
    $bmp = Get-CenteredIconBitmap $croppedBmp $sz $pad
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    $pngStreams += @{ Size = $sz; Bytes = $ms.ToArray() }
    $ms.Dispose()
}

$icoFile = Join-Path $iconsDir "icon.ico"
$fs = [System.IO.File]::Create($icoFile)
$bw = New-Object System.IO.BinaryWriter($fs)
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$pngStreams.Count)
$offset = 6 + (16 * $pngStreams.Count)
foreach ($entry in $pngStreams) {
    $w = if ($entry.Size -ge 256) { [byte]0 } else { [byte]$entry.Size }
    $h = if ($entry.Size -ge 256) { [byte]0 } else { [byte]$entry.Size }
    $bw.Write($w); $bw.Write($h); $bw.Write([byte]0); $bw.Write([byte]0)
    $bw.Write([uint16]1); $bw.Write([uint16]32)
    $bw.Write([uint32]$entry.Bytes.Length); $bw.Write([uint32]$offset)
    $offset += $entry.Bytes.Length
}
foreach ($entry in $pngStreams) { $bw.Write($entry.Bytes) }
$bw.Flush(); $bw.Close(); $fs.Close()
Write-Output "Saved centered multi-res $icoFile"

# 2. Generate Custom Branded Installer BMPs (24-bit RGB required by NSIS & WiX)
function Draw-SidebarPanel($g, $w, $h, $logo) {
    $rect = New-Object System.Drawing.Rectangle(0, 0, $w, $h)
    $bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $rect,
        [System.Drawing.Color]::FromArgb(255, 11, 14, 24),
        [System.Drawing.Color]::FromArgb(255, 18, 12, 34),
        90.0
    )
    $g.FillRectangle($bgBrush, $rect)
    $bgBrush.Dispose()

    # Subtle top accent bar
    $accentBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (New-Object System.Drawing.Rectangle(0, 0, $w, 4)),
        [System.Drawing.Color]::FromArgb(255, 168, 85, 247),
        [System.Drawing.Color]::FromArgb(255, 0, 240, 255),
        0.0
    )
    $g.FillRectangle($accentBrush, 0, 0, $w, 4)
    $accentBrush.Dispose()

    # Centered emblem
    $iconSz = 78
    $centered = Get-CenteredIconBitmap $logo $iconSz 0.04
    $g.DrawImage($centered, [int](($w - $iconSz) / 2), 68, $iconSz, $iconSz)
    $centered.Dispose()

    # Crisp title & tagline
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $titleFont = New-Object System.Drawing.Font("Segoe UI", 13, [System.Drawing.FontStyle]::Bold)
    $subFont = New-Object System.Drawing.Font("Segoe UI", 8, [System.Drawing.FontStyle]::Regular)
    $titleBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 248, 250, 252))
    $subBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 148, 163, 184))

    $g.DrawString("StreamShield", $titleFont, $titleBrush, [float]($w / 2.0), 160.0, $sf)
    $g.DrawString("Stream Privacy Manager", $subFont, $subBrush, [float]($w / 2.0), 186.0, $sf)

    $titleFont.Dispose(); $subFont.Dispose(); $titleBrush.Dispose(); $subBrush.Dispose(); $sf.Dispose()
}

# NSIS Sidebar (164 x 314)
$nsisSide = New-Object System.Drawing.Bitmap(164, 314, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($nsisSide)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
Draw-SidebarPanel $g 164 314 $croppedBmp
$g.Dispose()
$nsisSide.Save((Join-Path $iconsDir "nsis-sidebar.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$nsisSide.Dispose()

# NSIS Header (150 x 57)
$nsisHead = New-Object System.Drawing.Bitmap(150, 57, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($nsisHead)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g.Clear([System.Drawing.Color]::FromArgb(255, 11, 14, 24))
$accentH = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Rectangle(0, 54, 150, 3)),
    [System.Drawing.Color]::FromArgb(255, 168, 85, 247),
    [System.Drawing.Color]::FromArgb(255, 0, 240, 255),
    0.0
)
$g.FillRectangle($accentH, 0, 54, 150, 3)
$accentH.Dispose()
$hIcon = Get-CenteredIconBitmap $croppedBmp 36 0.04
$g.DrawImage($hIcon, 102, 9, 36, 36)
$hIcon.Dispose()
$g.Dispose()
$nsisHead.Save((Join-Path $iconsDir "nsis-header.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$nsisHead.Dispose()

# WiX Dialog (493 x 312 - dark branded left 164px sidebar + clean white right area for WiX black text)
$wixDlg = New-Object System.Drawing.Bitmap(493, 312, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($wixDlg)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
$g.Clear([System.Drawing.Color]::White)
Draw-SidebarPanel $g 164 312 $croppedBmp
$g.Dispose()
$wixDlg.Save((Join-Path $iconsDir "wix-dialog.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$wixDlg.Dispose()

# WiX Banner (493 x 58 - clean white left area for WiX black header text + branded right emblem)
$wixBan = New-Object System.Drawing.Bitmap(493, 58, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($wixBan)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g.Clear([System.Drawing.Color]::White)
$bAccent = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Rectangle(0, 55, 493, 3)),
    [System.Drawing.Color]::FromArgb(255, 168, 85, 247),
    [System.Drawing.Color]::FromArgb(255, 0, 240, 255),
    0.0
)
$g.FillRectangle($bAccent, 0, 55, 493, 3)
$bAccent.Dispose()
$bIcon = Get-CenteredIconBitmap $croppedBmp 38 0.04
$g.DrawImage($bIcon, 442, 9, 38, 38)
$bIcon.Dispose()
$g.Dispose()
$wixBan.Save((Join-Path $iconsDir "wix-banner.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$wixBan.Dispose()

$croppedBmp.Dispose()
Write-Output "Generated custom NSIS & WiX installer branding BMPs!"

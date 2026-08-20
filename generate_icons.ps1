Add-Type -AssemblyName System.Drawing

$srcPath = "C:\Users\GOYAL\.gemini\antigravity-ide\brain\31c474b2-1d56-4484-8e72-bd74eb3ba456\.user_uploaded\media_1787235438639.png"
$iconsDir = "C:\Users\GOYAL\Documents\work\StreamShield\tauri-app\src-tauri\icons"
$publicDir = "C:\Users\GOYAL\Documents\work\StreamShield\tauri-app\public"

if (!(Test-Path $iconsDir)) { New-Item -ItemType Directory -Path $iconsDir -Force }
if (!(Test-Path $publicDir)) { New-Item -ItemType Directory -Path $publicDir -Force }

$srcImg = [System.Drawing.Image]::FromFile($srcPath)

function Get-ResizedBitmap($img, $size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.DrawImage($img, 0, 0, $size, $size)
    $g.Dispose()
    return $bmp
}

# 1. Save standard PNGs
$sizes = @(
    @{ Name = "icon.png"; Size = 512 },
    @{ Name = "128x128.png"; Size = 128 },
    @{ Name = "128x128@2x.png"; Size = 256 },
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
    $resized = Get-ResizedBitmap $srcImg $s.Size
    $dest = Join-Path $iconsDir $s.Name
    $resized.Save($dest, [System.Drawing.Imaging.ImageFormat]::Png)
    $resized.Dispose()
    Write-Output "Saved $dest"
}

# Also save to public/logo.png for frontend UI
$logoResized = Get-ResizedBitmap $srcImg 256
$logoDest = Join-Path $publicDir "logo.png"
$logoResized.Save($logoDest, [System.Drawing.Imaging.ImageFormat]::Png)
$logoResized.Dispose()
Write-Output "Saved $logoDest"

# 2. Build multi-resolution icon.ico
$icoSizes = @(16, 24, 32, 48, 64, 128, 256)
$pngStreams = @()

foreach ($sz in $icoSizes) {
    $bmp = Get-ResizedBitmap $srcImg $sz
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    $pngStreams += @{ Size = $sz; Bytes = $ms.ToArray() }
    $ms.Dispose()
}

$srcImg.Dispose()

$icoFile = Join-Path $iconsDir "icon.ico"
$fs = [System.IO.File]::Create($icoFile)
$bw = New-Object System.IO.BinaryWriter($fs)

# ICONDIR header: Reserved (0), Type (1 for ICO), Count
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$pngStreams.Count)

$offset = 6 + (16 * $pngStreams.Count)

# ICONDIRENTRY for each image
foreach ($entry in $pngStreams) {
    $w = if ($entry.Size -ge 256) { [byte]0 } else { [byte]$entry.Size }
    $h = if ($entry.Size -ge 256) { [byte]0 } else { [byte]$entry.Size }
    $bw.Write($w)                    # bWidth
    $bw.Write($h)                    # bHeight
    $bw.Write([byte]0)               # bColorCount
    $bw.Write([byte]0)               # bReserved
    $bw.Write([uint16]1)             # wPlanes
    $bw.Write([uint16]32)            # wBitCount
    $bw.Write([uint32]$entry.Bytes.Length) # dwBytesInRes
    $bw.Write([uint32]$offset)       # dwImageOffset
    $offset += $entry.Bytes.Length
}

# Image data
foreach ($entry in $pngStreams) {
    $bw.Write($entry.Bytes)
}

$bw.Flush()
$bw.Close()
$fs.Close()

Write-Output "Saved $icoFile with multi-resolution frames!"

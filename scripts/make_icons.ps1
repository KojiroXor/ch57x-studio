Add-Type -AssemblyName System.Drawing

function Convert-ToIco {
    param(
        [Parameter(Mandatory=$true)][string]$sourcePath,
        [Parameter(Mandatory=$true)][string]$icoPath
    )

    $sizes = @(16, 24, 32, 48, 64, 128, 256)
    $resolvedSrc = (Resolve-Path $sourcePath).Path
    $srcImg = [System.Drawing.Bitmap]::FromFile($resolvedSrc)
    
    $pngStreams = @()
    foreach ($sz in $sizes) {
        $resized = New-Object System.Drawing.Bitmap $sz, $sz
        $g = [System.Drawing.Graphics]::FromImage($resized)
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $g.DrawImage($srcImg, 0, 0, $sz, $sz)
        $g.Dispose()

        $ms = New-Object System.IO.MemoryStream
        $resized.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $resized.Dispose()
        $pngStreams += ,@($sz, $ms.ToArray())
        $ms.Dispose()
    }
    $srcImg.Dispose()

    $parentDir = Split-Path $icoPath
    if ($parentDir -and -not (Test-Path $parentDir)) {
        New-Item -ItemType Directory -Force -Path $parentDir | Out-Null
    }

    $fs = [System.IO.File]::Create($icoPath)
    $bw = New-Object System.IO.BinaryWriter $fs

    # ICONDIR
    $bw.Write([uint16]0) # Reserved
    $bw.Write([uint16]1) # Type (1 = ICO)
    $bw.Write([uint16]$pngStreams.Count) # Image count

    $offset = 6 + ($pngStreams.Count * 16)
    foreach ($item in $pngStreams) {
        $sz = $item[0]
        $bytes = $item[1]
        
        $w = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }
        $h = if ($sz -ge 256) { [byte]0 } else { [byte]$sz }
        $bw.Write($w)
        $bw.Write($h)
        $bw.Write([byte]0) # Color palette count
        $bw.Write([byte]0) # Reserved
        $bw.Write([uint16]1) # Color planes
        $bw.Write([uint16]32) # Bits per pixel
        $bw.Write([uint32]$bytes.Length) # Image size
        $bw.Write([uint32]$offset) # Offset
        $offset += $bytes.Length
    }

    foreach ($item in $pngStreams) {
        $bytes = $item[1]
        $bw.Write($bytes)
    }

    $bw.Flush()
    $fs.Close()
    Write-Host "Created: $icoPath ($((Get-Item $icoPath).Length) bytes)"
}

function Export-Rgba {
    param(
        [Parameter(Mandatory=$true)][string]$sourcePath,
        [Parameter(Mandatory=$true)][string]$outPath,
        [Parameter(Mandatory=$true)][int]$size
    )

    $resolvedSrc = (Resolve-Path $sourcePath).Path
    $srcImg = [System.Drawing.Bitmap]::FromFile($resolvedSrc)
    $resized = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($resized)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.DrawImage($srcImg, 0, 0, $size, $size)
    $g.Dispose()
    $srcImg.Dispose()

    $rect = New-Object System.Drawing.Rectangle 0, 0, $size, $size
    $bmpData = $resized.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    
    $byteCount = $size * $size * 4
    $bgraBytes = New-Object byte[] $byteCount
    [System.Runtime.InteropServices.Marshal]::Copy($bmpData.Scan0, $bgraBytes, 0, $byteCount)
    $resized.UnlockBits($bmpData)
    $resized.Dispose()

    # Convert BGRA to RGBA for egui/eframe
    $rgbaBytes = New-Object byte[] $byteCount
    for ($i = 0; $i -lt $byteCount; $i += 4) {
        $rgbaBytes[$i]     = $bgraBytes[$i + 2] # R
        $rgbaBytes[$i + 1] = $bgraBytes[$i + 1] # G
        $rgbaBytes[$i + 2] = $bgraBytes[$i]     # B
        $rgbaBytes[$i + 3] = $bgraBytes[$i + 3] # A
    }

    [System.IO.File]::WriteAllBytes($outPath, $rgbaBytes)
    Write-Host "Exported RGBA: $outPath ($size x $size, $byteCount bytes)"
}

Convert-ToIco "assets\icons\vortexdeck_app_icon.png" "assets\icons\vortexdeck_app_icon.ico"
Convert-ToIco "assets\icons\vortexdeck_tray_icon.png" "assets\icons\vortexdeck_tray_icon.ico"
Convert-ToIco "assets\icons\vortexdeck_tray_paused.png" "assets\icons\vortexdeck_tray_paused.ico"

Export-Rgba "assets\icons\vortexdeck_app_icon.png" "assets\icons\vortexdeck_app_icon_64.rgba" 64
Export-Rgba "assets\icons\vortexdeck_tray_icon.png" "assets\icons\vortexdeck_tray_icon_32.rgba" 32
Export-Rgba "assets\icons\vortexdeck_tray_paused.png" "assets\icons\vortexdeck_tray_paused_32.rgba" 32

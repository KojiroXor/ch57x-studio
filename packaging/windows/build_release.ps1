# VortexDeck Windows Release Packager
param (
    [string]$Configuration = "release",
    [switch]$ForceRebuild = $false
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  VortexDeck - Windows Release Packager   " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.Parent.FullName
$TargetExe = Join-Path $WorkspaceRoot "target\$Configuration\vortexdeck.exe"
$DistDir = Join-Path $WorkspaceRoot "dist\VortexDeck"

if ($ForceRebuild -or (-not (Test-Path $TargetExe))) {
    Write-Host "[1/3] Compiling VortexDeck in $Configuration mode..." -ForegroundColor Yellow
    $env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
    & cargo build --release -p ch57x-studio
} else {
    Write-Host "[1/3] Using compiled binary at $TargetExe" -ForegroundColor Green
}

Write-Host "[2/3] Assembling distribution bundle into $DistDir..." -ForegroundColor Yellow
Get-Process vortexdeck -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

if (Test-Path $DistDir) {
    try {
        Remove-Item -Recurse -Force $DistDir -ErrorAction Stop
    } catch {
        Write-Host "Warning: Could not remove directory, cleaning files individually..." -ForegroundColor Yellow
        Get-ChildItem -Path $DistDir -Recurse | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
    }
}
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $DistDir "bin") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $DistDir "assets\icons") | Out-Null

Copy-Item $TargetExe -Destination (Join-Path $DistDir "vortexdeck.exe") -Force

Write-Host "Embedding application icon into vortexdeck.exe PE resource header..." -ForegroundColor Cyan
& python (Join-Path $WorkspaceRoot "scripts\inject_icon.py") (Join-Path $DistDir "vortexdeck.exe") (Join-Path $WorkspaceRoot "assets\icons\vortexdeck_app_icon.ico")
& python (Join-Path $WorkspaceRoot "scripts\inject_icon.py") $TargetExe (Join-Path $WorkspaceRoot "assets\icons\vortexdeck_app_icon.ico")

Copy-Item (Join-Path $WorkspaceRoot "bin\ch57x-keyboard-tool.exe") -Destination (Join-Path $DistDir "bin\ch57x-keyboard-tool.exe") -Force
Copy-Item (Join-Path $WorkspaceRoot "bin\example-mapping.yaml") -Destination (Join-Path $DistDir "bin\example-mapping.yaml") -Force
Copy-Item (Join-Path $WorkspaceRoot "assets\icons\*") -Destination (Join-Path $DistDir "assets\icons") -Recurse -Force

$readmeContent = @"
# VortexDeck (CH57x Studio)
Configuratore Visuale e Runtime Nativo di Automazione per Macropad CH57x (3 Layer, 54 Slot).

## Avvio Rapido
1. Esegui `vortexdeck.exe` per aprire l'interfaccia grafica.
2. Configura i tuoi 12 tasti e 2 encoder rotativi (3 layer = 54 slot).
3. Premi **"▶️ Applica Runtime (Host)"** per attivare le azioni Windows (Workspace, Macro, Finestre) senza scrivere sul dispositivo fisico.
4. Quando desideri programmare la memoria flash hardware del Macropad, collega il cavo USB e premi **"⚡ Scrivi Memoria Flash"**.

## Modalità Sfondo (Background Daemon)
Per eseguire VortexDeck all'avvio di Windows senza aprire la finestra grafica:
```powershell
.\vortexdeck.exe --background
```
VortexDeck rimarrà attivo nella System Tray (barra delle applicazioni vicino all'orologio).

## Diagnostica Rapida
```powershell
.\vortexdeck.exe --doctor
```

## Requisiti USB (per scrittura hardware flash)
Per consentire a `ch57x-keyboard-tool` di comunicare a basso livello con il chip CH57x via USB, è necessario installare UsbDk su Windows:
- UsbDk Installer: https://github.com/daynix/UsbDk/releases
"@

Set-Content -Path (Join-Path $DistDir "README.txt") -Value $readmeContent -Encoding UTF8

Write-Host "[3/3] Distribution bundle ready!" -ForegroundColor Green
Write-Host "Cartella distribuibile: $DistDir" -ForegroundColor Cyan
Get-ChildItem -Recurse $DistDir | Select-Object Name, Length

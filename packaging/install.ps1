#Requires -Version 5.1
<#
.SYNOPSIS
    Installs OpenATK (GUI) and vxectl (CLI) for the current Windows user.
.DESCRIPTION
    Copies open-atk.exe and vxectl.exe to %LOCALAPPDATA%\OpenATK\bin,
    adds that folder to the User PATH, and creates a Start Menu shortcut.
#>

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "OpenATK\bin"
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot   = Split-Path -Parent $ScriptDir
$TargetDir  = Join-Path $RepoRoot "target\release"

# Locate binaries: release archive (same dir) or cargo target\release
if (Test-Path (Join-Path $ScriptDir "open-atk.exe")) {
    $SourceDir = $ScriptDir
} elseif (Test-Path (Join-Path $TargetDir "open-atk.exe")) {
    $SourceDir = $TargetDir
} else {
    Write-Host "[*] Compiled binaries not found. Building with Cargo..." -ForegroundColor Cyan
    Push-Location $RepoRoot
    try {
        cargo build --release --workspace
        $SourceDir = $TargetDir
    } finally {
        Pop-Location
    }
}

$GuiBin = Join-Path $SourceDir "open-atk.exe"
$CliBin = Join-Path $SourceDir "vxectl.exe"

if (-not (Test-Path $GuiBin) -or -not (Test-Path $CliBin)) {
    Write-Error "Missing open-atk.exe or vxectl.exe in $SourceDir"
    exit 1
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -Path $GuiBin -Destination (Join-Path $InstallDir "open-atk.exe") -Force
Copy-Item -Path $CliBin -Destination (Join-Path $InstallDir "vxectl.exe") -Force
Write-Host "[+] Installed binaries to $InstallDir" -ForegroundColor Green

# Add to User PATH if missing
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$PathEntries = ($UserPath -split ";") | Where-Object { $_ -ne "" }
if ($PathEntries -notcontains $InstallDir) {
    $NewPath = ($PathEntries + $InstallDir) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "[+] Added $InstallDir to User PATH" -ForegroundColor Green
} else {
    Write-Host "[=] $InstallDir is already in User PATH" -ForegroundColor DarkGray
}

# Create Start Menu shortcut for OpenATK GUI
$StartMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$ShortcutPath = Join-Path $StartMenuDir "OpenATK.lnk"
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = Join-Path $InstallDir "open-atk.exe"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "OpenATK - Gaming Mouse Configuration Suite"
$Shortcut.Save()
Write-Host "[+] Created Start Menu shortcut: $ShortcutPath" -ForegroundColor Green

Write-Host ""
Write-Host "Installation complete! Open a new terminal and run 'vxectl' or launch OpenATK from the Start Menu." -ForegroundColor Cyan


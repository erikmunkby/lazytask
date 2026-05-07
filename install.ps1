#!/usr/bin/env pwsh
#
# lazytask installer for Windows (x86_64)
# Usage: irm https://raw.githubusercontent.com/erikmunkby/lazytask/main/install.ps1 | iex
#
# Environment variables:
#   LAZYTASK_VERSION     — pin a release tag (e.g. "lazytask-v0.6.1"); default: latest
#   LAZYTASK_INSTALL_DIR — override install directory; default: %LOCALAPPDATA%\lazytask\bin

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = "erikmunkby/lazytask"
$Target = "x86_64-pc-windows-msvc"

# --- Platform check -----------------------------------------------------------

$arch = $env:PROCESSOR_ARCHITECTURE
$wow64 = $env:PROCESSOR_ARCHITEW6432

$isX64 = ($arch -eq 'AMD64') -or ($arch -eq 'x86' -and $wow64 -eq 'AMD64')
if (-not $isX64) {
    Write-Host "Error: lazytask pre-built binaries require 64-bit (x86_64) Windows." -ForegroundColor Red
    Write-Host "Detected architecture: $arch"
    Write-Host "Use 'cargo install lazytask' to build from source for this platform."
    exit 1
}

# --- Resolve version ----------------------------------------------------------

if ($env:LAZYTASK_VERSION) {
    $Tag = $env:LAZYTASK_VERSION
    Write-Host "Using pinned version: $Tag"
} else {
    Write-Host "Resolving latest release..."
    try {
        $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
        $Tag = $release.tag_name
    } catch {
        Write-Host "Error: failed to fetch latest release from GitHub." -ForegroundColor Red
        Write-Host $_.Exception.Message
        exit 1
    }
    Write-Host "Latest release: $Tag"
}

$Version = $Tag -replace '^lazytask-v', ''

# --- Download -----------------------------------------------------------------

$AssetName = "lt-$Target-$Tag.zip"
$BaseUrl = "https://github.com/$Repo/releases/download/$Tag"
$TempDir = Join-Path $env:TEMP "lazytask-install-$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TempDir | Out-Null

$ZipPath = Join-Path $TempDir $AssetName
$SumsPath = Join-Path $TempDir "SHA256SUMS"

try {
    Write-Host "Downloading $AssetName..."
    Invoke-WebRequest "$BaseUrl/$AssetName" -OutFile $ZipPath -UseBasicParsing

    Write-Host "Downloading SHA256SUMS..."
    Invoke-WebRequest "$BaseUrl/SHA256SUMS" -OutFile $SumsPath -UseBasicParsing
} catch {
    Write-Host "Error: download failed." -ForegroundColor Red
    Write-Host $_.Exception.Message
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    exit 1
}

# --- Verify checksum ----------------------------------------------------------

$expectedHash = $null
foreach ($line in (Get-Content $SumsPath)) {
    $parts = $line -split '\s+', 2
    if ($parts.Length -eq 2 -and $parts[1].TrimStart('*') -eq $AssetName) {
        $expectedHash = $parts[0].ToUpper()
        break
    }
}

if (-not $expectedHash) {
    Write-Host "Error: could not find checksum for $AssetName in SHA256SUMS." -ForegroundColor Red
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    exit 1
}

$actualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash
if ($actualHash -ne $expectedHash) {
    Write-Host "Error: SHA256 mismatch!" -ForegroundColor Red
    Write-Host "  Expected: $expectedHash"
    Write-Host "  Actual:   $actualHash"
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "Checksum verified."

# --- Extract ------------------------------------------------------------------

$ExtractDir = Join-Path $TempDir "extracted"
Expand-Archive -Path $ZipPath -DestinationPath $ExtractDir -Force

$ExeFile = Get-ChildItem -Path $ExtractDir -Recurse -Filter 'lt.exe' | Select-Object -First 1
if (-not $ExeFile) {
    Write-Host "Error: lt.exe not found in archive." -ForegroundColor Red
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    exit 1
}

# --- Install ------------------------------------------------------------------

$InstallDir = if ($env:LAZYTASK_INSTALL_DIR) { $env:LAZYTASK_INSTALL_DIR } else {
    Join-Path $env:LOCALAPPDATA "lazytask\bin"
}

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$Dest = Join-Path $InstallDir "lt.exe"
Move-Item -Path $ExeFile.FullName -Destination $Dest -Force
Unblock-File -Path $Dest

Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue

# --- Update PATH --------------------------------------------------------------

$regKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
try {
    $currentPath = $regKey.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    $entries = $currentPath -split ';' | Where-Object { $_ -ne '' }
    $alreadyInPath = $entries | Where-Object { $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\') }

    if (-not $alreadyInPath) {
        $newPath = if ($currentPath -and -not $currentPath.EndsWith(';')) {
            "$currentPath;$InstallDir"
        } elseif ($currentPath) {
            "$currentPath$InstallDir"
        } else {
            $InstallDir
        }
        $regKey.SetValue('Path', $newPath, [Microsoft.Win32.RegistryValueKind]::ExpandString)

        # Broadcast WM_SETTINGCHANGE so new shells pick up the change
        if (-not ([System.Management.Automation.PSTypeName]'NativeMethods').Type) {
            Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @'
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
'@
        }
        $HWND_BROADCAST = [IntPtr]0xFFFF
        $WM_SETTINGCHANGE = 0x001A
        $SMTO_ABORTIFHUNG = 0x0002
        $result = [UIntPtr]::Zero
        [Win32.NativeMethods]::SendMessageTimeout(
            $HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero,
            'Environment', $SMTO_ABORTIFHUNG, 5000, [ref]$result
        ) | Out-Null
    }
} finally {
    $regKey.Close()
}

# Add to current session so lt works immediately
if ($env:Path -split ';' | Where-Object { $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\') }) {
    # already present
} else {
    $env:Path = "$InstallDir;$env:Path"
}

# --- Done ---------------------------------------------------------------------

Write-Host ""
$ltVersion = & $Dest --version 2>&1
Write-Host "lazytask $ltVersion installed to $Dest" -ForegroundColor Green
Write-Host ""
Write-Host "Open a new terminal to use 'lt' (already-open terminals need to be restarted)."
Write-Host ""
Write-Host "Quick start:"
Write-Host "  lt init    # set up .tasks/ in your project"
Write-Host "  lt         # open the TUI"

# StrayMark CLI installer for Windows — https://github.com/StrangeDaysTech/straymark
#
# Usage:
#   irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
#
#   # Or with parameters:
#   & ([scriptblock]::Create((irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1))) -Tag cli-1.0.0
#
# Compatible with PowerShell 5.1+ and PowerShell Core (pwsh).

param(
    [string]$Tag,
    [string]$InstallDir
)

$ErrorActionPreference = "Stop"

$Repo = "StrangeDaysTech/straymark"
$Binary = "straymark.exe"
$Target = "x86_64-pc-windows-msvc"

function Write-Status($Message) {
    Write-Host "straymark-install: $Message"
}

function Write-Err($Message) {
    Write-Host "straymark-install: ERROR: $Message" -ForegroundColor Red
}

# ── Verify platform ─────────────────────────────────────────────────────

if ([System.Environment]::Is64BitOperatingSystem -eq $false) {
    Write-Err "StrayMark requires a 64-bit Windows installation."
    exit 1
}

if ($env:PROCESSOR_ARCHITECTURE -ne "AMD64" -and $env:PROCESSOR_ARCHITECTURE -ne "x86") {
    # ARM64 Windows is not supported
    Write-Err "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE. Only x86_64 (AMD64) is supported."
    exit 1
}

# ── Defaults ─────────────────────────────────────────────────────────────

if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "StrayMark\bin"
}

# ── GitHub API headers ───────────────────────────────────────────────────

function Get-GitHubHeaders {
    $headers = @{
        "Accept" = "application/vnd.github+json"
    }
    if ($env:GITHUB_TOKEN) {
        $headers["Authorization"] = "token $env:GITHUB_TOKEN"
    }
    return $headers
}

# ── Get latest tag ───────────────────────────────────────────────────────

function Get-LatestTag {
    $apiUrl = "https://api.github.com/repos/$Repo/releases"
    $headers = Get-GitHubHeaders

    Write-Status "fetching latest CLI release info..."

    try {
        $releases = Invoke-RestMethod -Uri $apiUrl -Headers $headers -UseBasicParsing
    }
    catch {
        Write-Err "Failed to fetch release info from GitHub API."
        Write-Host ""
        Write-Host "  This may be due to rate limiting." -ForegroundColor Yellow
        Write-Host "  Set `$env:GITHUB_TOKEN to authenticate, or use -Tag to specify a version."
        Write-Host ""
        exit 1
    }

    $tagName = ($releases | Where-Object { $_.tag_name -like "cli-*" } | Select-Object -First 1).tag_name
    if (-not $tagName) {
        Write-Err "Could not find a CLI release (cli-* tag) from GitHub API response."
        exit 1
    }

    Write-Status "latest version: $tagName"
    return $tagName
}

# ── Main ─────────────────────────────────────────────────────────────────

$tempDir = $null

try {
    # Resolve tag
    if (-not $Tag) {
        $Tag = Get-LatestTag
    }
    else {
        Write-Status "using specified version: $Tag"
    }

    # Build asset name and URL
    $versionNum = $Tag -replace "^cli-", ""
    $asset = "straymark-cli-v${versionNum}-${Target}.zip"
    $url = "https://github.com/$Repo/releases/download/$Tag/$asset"

    # Create temp directory
    $tempDir = Join-Path $env:TEMP "straymark-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    $archivePath = Join-Path $tempDir $asset

    # Download
    Write-Status "downloading $asset..."
    try {
        # Use TLS 1.2+ (required for GitHub)
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

        $headers = Get-GitHubHeaders
        Invoke-WebRequest -Uri $url -OutFile $archivePath -Headers $headers -UseBasicParsing
    }
    catch {
        Write-Err "Download failed."
        Write-Host ""
        Write-Host "  Possible causes:" -ForegroundColor Yellow
        Write-Host "  - Version $Tag does not exist"
        Write-Host "  - No binary available for $Target"
        Write-Host "  - Network connectivity issue"
        Write-Host ""
        exit 1
    }

    # Extract
    Write-Status "extracting $Binary..."
    $extractDir = Join-Path $tempDir "extract"
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

    $binaryPath = Join-Path $extractDir $Binary
    if (-not (Test-Path $binaryPath)) {
        Write-Err "Binary '$Binary' not found in archive."
        exit 1
    }

    # Install
    if (-not (Test-Path $InstallDir)) {
        try {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        catch {
            Write-Err "Failed to create install directory: $InstallDir"
            Write-Host ""
            Write-Host "  Try specifying an alternative with -InstallDir" -ForegroundColor Yellow
            Write-Host ""
            exit 1
        }
    }

    $destPath = Join-Path $InstallDir $Binary
    Copy-Item -Path $binaryPath -Destination $destPath -Force
    Write-Status "installed $Binary to $destPath"

    # Add to PATH if not already present
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = $userPath -split ";"

    if ($pathEntries -notcontains $InstallDir) {
        Write-Status "adding $InstallDir to user PATH..."
        $newPath = "$userPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")

        # Also update current session PATH
        $env:Path = "$env:Path;$InstallDir"
        Write-Host ""
        Write-Host "  $InstallDir has been added to your user PATH." -ForegroundColor Green
        Write-Host "  Restart your terminal for the change to take effect in new sessions."
        Write-Host ""
    }

    # Verify
    try {
        $version = & $destPath --version 2>&1
        Write-Status "verified: $version"
    }
    catch {
        Write-Status "warning: could not verify installation (binary may not run on this platform)"
    }

    Write-Status "done!"
}
finally {
    # Cleanup temp directory
    if ($tempDir -and (Test-Path $tempDir)) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

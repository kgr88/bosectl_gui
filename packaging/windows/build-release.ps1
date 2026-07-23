[CmdletBinding()]
param(
    [string]$PythonExecutable = "python",
    [string]$TargetTriple = "x86_64-pc-windows-msvc",
    [switch]$SkipTests,
    [string]$CertificateThumbprint = $env:BOSECTL_CERTIFICATE_THUMBPRINT,
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($env:OS -ne "Windows_NT") {
    throw "The release pipeline must run on Windows."
}
if ($TargetTriple -notmatch '^[A-Za-z0-9_.-]+-pc-windows-msvc$') {
    throw "TargetTriple must be a Windows MSVC Rust target triple."
}

$repository = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$buildRoot = Join-Path $PSScriptRoot ".build"
$pyinstallerDist = Join-Path $buildRoot "dist"
$pyinstallerWork = Join-Path $buildRoot "pyinstaller"
$sidecarSource = Join-Path $pyinstallerDist "bosectl-backend"
$sidecarBinary = Join-Path $sidecarSource "bosectl-backend.exe"
$sidecarLibraries = Join-Path $sidecarSource "bosectl-backend-lib"
$tauriBinaries = Join-Path $repository "desktop/src-tauri/binaries"
$stagedBinary = Join-Path $tauriBinaries ("bosectl-backend-{0}.exe" -f $TargetTriple)
$stagedLibraries = Join-Path $tauriBinaries "bosectl-backend-lib"

foreach ($path in @($buildRoot, $tauriBinaries)) {
    $fullPath = [System.IO.Path]::GetFullPath($path)
    if (-not $fullPath.StartsWith($repository, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to use a build path outside the repository: $fullPath"
    }
}

if (Test-Path -LiteralPath $buildRoot) {
    Remove-Item -LiteralPath $buildRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $pyinstallerDist, $pyinstallerWork -Force | Out-Null

& $PythonExecutable -m PyInstaller --noconfirm --clean `
    --distpath $pyinstallerDist --workpath $pyinstallerWork `
    (Join-Path $PSScriptRoot "pyinstaller.spec")
if ($LASTEXITCODE -ne 0) { throw "PyInstaller failed." }
if (-not (Test-Path -LiteralPath $sidecarBinary -PathType Leaf)) {
    throw "PyInstaller did not produce $sidecarBinary"
}
if (-not (Test-Path -LiteralPath $sidecarLibraries -PathType Container)) {
    throw "PyInstaller did not produce its one-folder support directory."
}

& $PythonExecutable (Join-Path $PSScriptRoot "smoke_sidecar.py") $sidecarBinary
if ($LASTEXITCODE -ne 0) { throw "The frozen backend smoke test failed." }

New-Item -ItemType Directory -Path $tauriBinaries -Force | Out-Null
if (Test-Path -LiteralPath $stagedBinary) {
    Remove-Item -LiteralPath $stagedBinary -Force
}
if (Test-Path -LiteralPath $stagedLibraries) {
    Remove-Item -LiteralPath $stagedLibraries -Recurse -Force
}
Copy-Item -LiteralPath $sidecarBinary -Destination $stagedBinary
Copy-Item -LiteralPath $sidecarLibraries -Destination $stagedLibraries -Recurse

if (-not $SkipTests) {
    Push-Location (Join-Path $repository "python")
    try {
        & $PythonExecutable -m pytest
        if ($LASTEXITCODE -ne 0) { throw "Python tests failed." }
    } finally { Pop-Location }

    Push-Location (Join-Path $repository "desktop")
    try {
        & corepack pnpm test
        if ($LASTEXITCODE -ne 0) { throw "Renderer tests failed." }
    } finally { Pop-Location }

    Push-Location (Join-Path $repository "desktop/src-tauri")
    try {
        & cargo test
        if ($LASTEXITCODE -ne 0) { throw "Rust tests failed." }
    } finally { Pop-Location }
}

$releaseConfigPath = Join-Path $repository "desktop/src-tauri/tauri.release.conf.json"
$effectiveConfigPath = $releaseConfigPath
if ($CertificateThumbprint) {
    $signingConfig = @{
        bundle = @{
            active = $true
            externalBin = @("binaries/bosectl-backend")
            resources = @{
                "binaries/bosectl-backend-lib/" = "bosectl-backend-lib/"
            }
            windows = @{
                certificateThumbprint = $CertificateThumbprint
                digestAlgorithm = "sha256"
                timestampUrl = $TimestampUrl
            }
        }
    }
    $signingConfigPath = Join-Path $buildRoot "signing.conf.json"
    $signingConfig | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $signingConfigPath -Encoding UTF8
    $effectiveConfigPath = $signingConfigPath
} else {
    Write-Warning "No BOSECTL_CERTIFICATE_THUMBPRINT was supplied; release artifacts will be unsigned."
}
$tauriArguments = @(
    "pnpm", "tauri", "build", "--target", $TargetTriple,
    "--config", $effectiveConfigPath
)

Push-Location (Join-Path $repository "desktop")
try {
    & corepack @tauriArguments
    if ($LASTEXITCODE -ne 0) { throw "Tauri release build failed." }
} finally { Pop-Location }

$bundleRoot = Join-Path $repository ("desktop/src-tauri/target/{0}/release/bundle/nsis" -f $TargetTriple)
$installer = Get-ChildItem -LiteralPath $bundleRoot -Filter "*.exe" -File |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1
if (-not $installer) {
    throw "No NSIS installer was produced under $bundleRoot"
}
$checksum = Get-FileHash -Algorithm SHA256 -LiteralPath $installer.FullName
$checksumLine = "{0}  {1}" -f $checksum.Hash.ToLowerInvariant(), $installer.Name
$checksumPath = Join-Path $bundleRoot ($installer.Name + ".sha256")
$checksumLine | Set-Content -LiteralPath $checksumPath -Encoding ascii

Write-Host "Installer: $($installer.FullName)"
Write-Host "SHA-256:  $checksumPath"

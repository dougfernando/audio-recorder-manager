# Change to script directory
Set-Location $PSScriptRoot

# Remove Git's usr/bin from PATH temporarily
$env:PATH = ($env:PATH -split ';' | Where-Object { $_ -notlike '*Git\usr\bin*' }) -join ';'

# Clean previous build artifacts
Write-Host "Cleaning previous build..." -ForegroundColor Yellow
cargo clean

# Build the project using GNU toolchain
Write-Host "Building audio-recorder-manager with GNU toolchain..." -ForegroundColor Cyan
cargo +stable-x86_64-pc-windows-gnu build --release

# Check if build succeeded
if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild successful!" -ForegroundColor Green
    Write-Host "Executable location: target\release\audio-recorder-manager.exe"
} else {
    Write-Host "`nBuild failed with exit code: $LASTEXITCODE" -ForegroundColor Red
}

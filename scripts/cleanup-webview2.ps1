# Remove WebView2Loader.dll from release builds
# This DLL is not needed when using system WebView2 runtime

$releasePath = "target\x86_64-pc-windows-gnu\release"

if (Test-Path "$releasePath\WebView2Loader.dll") {
    Write-Host "Removing WebView2Loader.dll from release build..."
    Remove-Item "$releasePath\WebView2Loader.dll" -Force
    Write-Host "WebView2Loader.dll removed successfully"
} else {
    Write-Host "WebView2Loader.dll not found in release build"
}

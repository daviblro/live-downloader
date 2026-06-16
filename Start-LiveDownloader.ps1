$appPath = Join-Path $PSScriptRoot "LiveDownloader.App.ps1"

Start-Process `
    -FilePath "powershell.exe" `
    -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $appPath) `
    -WorkingDirectory $PSScriptRoot

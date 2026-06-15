$scriptPath = Join-Path $PSScriptRoot "LiveDownloader.ps1"

Start-Process `
    -FilePath "powershell.exe" `
    -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-File", $scriptPath) `
    -WorkingDirectory $PSScriptRoot `
    -WindowStyle Hidden

$configPath = Join-Path $PSScriptRoot "config.json"
$streamUrls = @()

if (Test-Path -LiteralPath $configPath) {
    try {
        $config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
        $streamUrls = @($config.Streams | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    }
    catch {
        Write-Host "Could not read config.json. Stopping worker processes only."
    }
}

$workerProcesses = Get-CimInstance Win32_Process |
    Where-Object {
        $_.CommandLine -match "LiveDownloader\.ps1" -and
        $_.CommandLine -match [regex]::Escape($PSScriptRoot)
    }

$downloadProcesses = @()
if ($streamUrls.Count -gt 0) {
    $downloadProcesses = Get-CimInstance Win32_Process |
        Where-Object {
            $processCommandLine = $_.CommandLine
            $_.Name -match "yt-dlp" -and
            ($streamUrls | Where-Object { -not [string]::IsNullOrWhiteSpace($processCommandLine) -and $processCommandLine.Contains($_) }).Count -gt 0
        }
}

$processes = @()
if ($null -ne $workerProcesses) {
    $processes += @($workerProcesses)
}
if ($null -ne $downloadProcesses) {
    $processes += @($downloadProcesses)
}

foreach ($process in $processes) {
    Stop-Process -Id $process.ProcessId -Force
    Write-Host "Stopped process $($process.ProcessId): $($process.Name)"
}

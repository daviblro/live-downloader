$escapedUrl = "https://www.twitch.tv/soucarlosdaniel"

$workerProcesses = Get-CimInstance Win32_Process |
    Where-Object {
        $_.CommandLine -match "LiveDownloader\.ps1" -and
        $_.CommandLine -match [regex]::Escape($PSScriptRoot)
    }

$downloadProcesses = Get-CimInstance Win32_Process |
    Where-Object {
        $_.Name -ieq "yt-dlp_x86.exe" -and
        $_.CommandLine -like "*$escapedUrl*"
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

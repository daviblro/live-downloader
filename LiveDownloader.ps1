param(
    [string[]]$StreamUrl = @(),
    [string]$ConfigPath = (Join-Path $PSScriptRoot "config.json"),
    [string]$YtDlpPath = "C:\ffmpeg\yt-dlp_x86.exe",
    [int]$CheckIntervalSeconds = 300,
    [string]$DownloadDirectory = (Join-Path $PSScriptRoot "downloads"),
    [string]$LogDirectory = (Join-Path $PSScriptRoot "logs")
)

$ErrorActionPreference = "Stop"

$script:MainLogPath = Join-Path $LogDirectory "live-downloader.log"
$script:StatusPath = Join-Path $LogDirectory "status.json"
$script:Downloads = @{}
$script:LastProbeAt = @{}

function New-DirectoryIfMissing {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
}

function Write-Log {
    param([string]$Message)

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $line = "[$timestamp] $Message"
    Add-Content -LiteralPath $script:MainLogPath -Value $line
    Write-Host $line
}

function Get-DefaultConfig {
    return [ordered]@{
        Streams = @("https://www.twitch.tv/soucarlosdaniel")
        YtDlpPath = "C:\ffmpeg\yt-dlp_x86.exe"
        CheckIntervalSeconds = 300
        DownloadDirectory = (Join-Path $PSScriptRoot "downloads")
        LogDirectory = (Join-Path $PSScriptRoot "logs")
    }
}

function Save-Config {
    param($Config)

    $Config | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $ConfigPath -Encoding UTF8
}

function Get-Config {
    $config = Get-DefaultConfig

    if (Test-Path -LiteralPath $ConfigPath) {
        try {
            $saved = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json

            if ($null -ne $saved.Streams) {
                $config.Streams = @($saved.Streams | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)
            }
            if (-not [string]::IsNullOrWhiteSpace($saved.YtDlpPath)) {
                $config.YtDlpPath = [string]$saved.YtDlpPath
            }
            if ($saved.CheckIntervalSeconds -as [int]) {
                $config.CheckIntervalSeconds = [int]$saved.CheckIntervalSeconds
            }
            if (-not [string]::IsNullOrWhiteSpace($saved.DownloadDirectory)) {
                $config.DownloadDirectory = [string]$saved.DownloadDirectory
            }
            if (-not [string]::IsNullOrWhiteSpace($saved.LogDirectory)) {
                $config.LogDirectory = [string]$saved.LogDirectory
            }
        }
        catch {
            Write-Log "Could not read config. Error: $($_.Exception.Message)"
        }
    }
    else {
        if ($StreamUrl.Count -gt 0) {
            $config.Streams = @($StreamUrl | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)
        }
        $config.YtDlpPath = $YtDlpPath
        $config.CheckIntervalSeconds = $CheckIntervalSeconds
        $config.DownloadDirectory = $DownloadDirectory
        $config.LogDirectory = $LogDirectory
        Save-Config -Config $config
    }

    if ($config.CheckIntervalSeconds -lt 30) {
        $config.CheckIntervalSeconds = 30
    }

    return $config
}

function Get-SafeId {
    param([string]$Value)

    $md5 = [System.Security.Cryptography.MD5]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Value)
        $hash = $md5.ComputeHash($bytes)
        return -join ($hash | ForEach-Object { $_.ToString("x2") })
    }
    finally {
        $md5.Dispose()
    }
}

function Test-StreamLive {
    param(
        [string]$Url,
        $Config
    )

    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $safeId = Get-SafeId -Value $Url
    $probeLog = Join-Path $Config.LogDirectory "probe-$safeId-$stamp.log"

    & $Config.YtDlpPath --no-cache --simulate --quiet --no-warnings $Url *> $probeLog
    $exitCode = $LASTEXITCODE
    $script:LastProbeAt[$Url] = Get-Date

    if ($exitCode -eq 0) {
        Write-Log "Probe succeeded; stream appears live. Url=$Url"
        return $true
    }

    Write-Log "Probe did not find a live stream. Exit code: $exitCode Url=$Url Details=$probeLog"
    return $false
}

function Start-StreamDownload {
    param(
        [string]$Url,
        $Config
    )

    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $safeId = Get-SafeId -Value $Url
    $stdoutPath = Join-Path $Config.LogDirectory "download-$safeId-$stamp.out.log"
    $stderrPath = Join-Path $Config.LogDirectory "download-$safeId-$stamp.err.log"
    $argumentList = @("--no-cache", $Url)

    Write-Log "Starting download. Url=$Url Output=$stdoutPath Error=$stderrPath"

    return Start-Process `
        -FilePath $Config.YtDlpPath `
        -ArgumentList $argumentList `
        -WorkingDirectory $Config.DownloadDirectory `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath `
        -WindowStyle Hidden `
        -PassThru
}

function Stop-RemovedStreams {
    param([string[]]$CurrentStreams)

    foreach ($url in @($script:Downloads.Keys)) {
        if ($CurrentStreams -notcontains $url) {
            $entry = $script:Downloads[$url]
            if ($null -ne $entry.Process -and -not $entry.Process.HasExited) {
                Stop-Process -Id $entry.Process.Id -Force
                Write-Log "Stopped download for removed stream. Url=$url ProcessId=$($entry.Process.Id)"
            }
            $script:Downloads.Remove($url)
        }
    }
}

function Write-Status {
    param($Config)

    $items = foreach ($url in $Config.Streams) {
        $entry = $script:Downloads[$url]
        $process = if ($null -ne $entry) { $entry.Process } else { $null }
        $state = "Watching"
        $processId = $null
        $lastMessage = "Waiting for live stream"

        if ($null -ne $process) {
            if ($process.HasExited) {
                $state = "Finished"
                $lastMessage = "Download exited with code $($process.ExitCode)"
            }
            else {
                $state = "Downloading"
                $processId = $process.Id
                $lastMessage = "Download running"
            }
        }

        [ordered]@{
            Url = $url
            State = $state
            ProcessId = $processId
            LastProbeAt = if ($script:LastProbeAt.ContainsKey($url)) { $script:LastProbeAt[$url].ToString("s") } else { $null }
            LastMessage = $lastMessage
        }
    }

    [ordered]@{
        UpdatedAt = (Get-Date).ToString("s")
        WorkerProcessId = $PID
        CheckIntervalSeconds = $Config.CheckIntervalSeconds
        Streams = @($items)
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $script:StatusPath -Encoding UTF8
}

New-DirectoryIfMissing -Path $DownloadDirectory
New-DirectoryIfMissing -Path $LogDirectory

$initialConfig = Get-Config
New-DirectoryIfMissing -Path $initialConfig.DownloadDirectory
New-DirectoryIfMissing -Path $initialConfig.LogDirectory

$script:MainLogPath = Join-Path $initialConfig.LogDirectory "live-downloader.log"
$script:StatusPath = Join-Path $initialConfig.LogDirectory "status.json"

Write-Log "Live downloader worker started. ConfigPath=$ConfigPath"

while ($true) {
    try {
        $config = Get-Config
        New-DirectoryIfMissing -Path $config.DownloadDirectory
        New-DirectoryIfMissing -Path $config.LogDirectory
        $script:MainLogPath = Join-Path $config.LogDirectory "live-downloader.log"
        $script:StatusPath = Join-Path $config.LogDirectory "status.json"

        if (-not (Test-Path -LiteralPath $config.YtDlpPath)) {
            Write-Log "yt-dlp executable not found at '$($config.YtDlpPath)'."
            Write-Status -Config $config
            Start-Sleep -Seconds $config.CheckIntervalSeconds
            continue
        }

        Stop-RemovedStreams -CurrentStreams $config.Streams

        foreach ($url in $config.Streams) {
            if ([string]::IsNullOrWhiteSpace($url)) {
                continue
            }

            $entry = $script:Downloads[$url]

            if ($null -ne $entry) {
                if ($entry.Process.HasExited) {
                    Write-Log "Download process exited. Url=$url ExitCode=$($entry.Process.ExitCode)"
                    $entry.Process.Dispose()
                    $script:Downloads.Remove($url)
                }
                else {
                    Write-Log "Download already running. Url=$url ProcessId=$($entry.Process.Id)"
                    continue
                }
            }

            if (Test-StreamLive -Url $url -Config $config) {
                $process = Start-StreamDownload -Url $url -Config $config
                $script:Downloads[$url] = @{
                    Process = $process
                    StartedAt = Get-Date
                }
                Write-Log "Download process started. Url=$url ProcessId=$($process.Id)"
            }
        }

        Write-Status -Config $config
        Start-Sleep -Seconds $config.CheckIntervalSeconds
    }
    catch {
        Write-Log "Error: $($_.Exception.Message)"
        Start-Sleep -Seconds 30
    }
}

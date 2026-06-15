param(
    [string]$StreamUrl = "https://www.twitch.tv/soucarlosdaniel",
    [string]$YtDlpPath = "C:\ffmpeg\yt-dlp_x86.exe",
    [int]$CheckIntervalSeconds = 300,
    [string]$DownloadDirectory = (Join-Path $PSScriptRoot "downloads"),
    [string]$LogDirectory = (Join-Path $PSScriptRoot "logs")
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message)

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $line = "[$timestamp] $Message"
    Add-Content -LiteralPath $script:MainLogPath -Value $line
    Write-Host $line
}

function New-DirectoryIfMissing {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
}

function Test-StreamLive {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $probeLog = Join-Path $LogDirectory "probe-$stamp.log"

    & $YtDlpPath --no-cache --simulate --quiet --no-warnings $StreamUrl *> $probeLog
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Log "Probe succeeded; stream appears live."
        return $true
    }

    Write-Log "Probe did not find a live stream. Exit code: $exitCode. Details: $probeLog"
    return $false
}

function Start-StreamDownload {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $stdoutPath = Join-Path $LogDirectory "download-$stamp.out.log"
    $stderrPath = Join-Path $LogDirectory "download-$stamp.err.log"
    $argumentList = @("--no-cache", $StreamUrl)

    Write-Log "Starting download with $YtDlpPath. Output: $stdoutPath Error: $stderrPath"

    return Start-Process `
        -FilePath $YtDlpPath `
        -ArgumentList $argumentList `
        -WorkingDirectory $DownloadDirectory `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath `
        -WindowStyle Hidden `
        -PassThru
}

New-DirectoryIfMissing -Path $DownloadDirectory
New-DirectoryIfMissing -Path $LogDirectory

$script:MainLogPath = Join-Path $LogDirectory "live-downloader.log"

if (-not (Test-Path -LiteralPath $YtDlpPath)) {
    throw "yt-dlp executable not found at '$YtDlpPath'."
}

if ($CheckIntervalSeconds -lt 30) {
    throw "CheckIntervalSeconds must be at least 30 seconds."
}

Write-Log "Live downloader started. StreamUrl=$StreamUrl CheckIntervalSeconds=$CheckIntervalSeconds DownloadDirectory=$DownloadDirectory"

$downloadProcess = $null

while ($true) {
    try {
        if ($null -ne $downloadProcess) {
            if ($downloadProcess.HasExited) {
                Write-Log "Download process exited with code $($downloadProcess.ExitCode)."
                $downloadProcess.Dispose()
                $downloadProcess = $null
            }
            else {
                Write-Log "Download already running. ProcessId=$($downloadProcess.Id)"
            }
        }

        if ($null -eq $downloadProcess) {
            if (Test-StreamLive) {
                $downloadProcess = Start-StreamDownload
                Write-Log "Download process started. ProcessId=$($downloadProcess.Id)"
            }
        }
    }
    catch {
        Write-Log "Error: $($_.Exception.Message)"
    }

    Start-Sleep -Seconds $CheckIntervalSeconds
}

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$script:ConfigPath = Join-Path $PSScriptRoot "config.json"
$script:DefaultDownloadDirectory = Join-Path $PSScriptRoot "downloads"
$script:DefaultLogDirectory = Join-Path $PSScriptRoot "logs"
$script:StatusPath = Join-Path $script:DefaultLogDirectory "status.json"

function New-DirectoryIfMissing {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
}

function Get-DefaultConfig {
    return [ordered]@{
        Streams = @("https://www.twitch.tv/soucarlosdaniel")
        YtDlpPath = "C:\ffmpeg\yt-dlp_x86.exe"
        CheckIntervalSeconds = 300
        DownloadDirectory = $script:DefaultDownloadDirectory
        LogDirectory = $script:DefaultLogDirectory
    }
}

function Get-AppConfig {
    $config = Get-DefaultConfig

    if (Test-Path -LiteralPath $script:ConfigPath) {
        try {
            $saved = Get-Content -LiteralPath $script:ConfigPath -Raw | ConvertFrom-Json
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
            [System.Windows.Forms.MessageBox]::Show("Could not read config.json. Defaults will be used.`r`n$($_.Exception.Message)", "Live Downloader", "OK", "Warning") | Out-Null
        }
    }

    if ($config.CheckIntervalSeconds -lt 30) {
        $config.CheckIntervalSeconds = 30
    }

    return $config
}

function Save-AppConfig {
    param($Config)

    New-DirectoryIfMissing -Path $Config.DownloadDirectory
    New-DirectoryIfMissing -Path $Config.LogDirectory
    $Config | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $script:ConfigPath -Encoding UTF8
    $script:StatusPath = Join-Path $Config.LogDirectory "status.json"
}

function Get-WorkerProcesses {
    $scriptPath = Join-Path $PSScriptRoot "LiveDownloader.ps1"
    $escapedScript = [regex]::Escape($scriptPath)

    return @(Get-CimInstance Win32_Process |
        Where-Object {
            $_.CommandLine -match $escapedScript -or
            ($_.CommandLine -match "LiveDownloader\.ps1" -and $_.CommandLine -match [regex]::Escape($PSScriptRoot))
        })
}

function Start-Worker {
    if ((Get-WorkerProcesses).Count -gt 0) {
        return
    }

    $scriptPath = Join-Path $PSScriptRoot "LiveDownloader.ps1"

    Start-Process `
        -FilePath "powershell.exe" `
        -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-File", $scriptPath, "-ConfigPath", $script:ConfigPath) `
        -WorkingDirectory $PSScriptRoot `
        -WindowStyle Hidden | Out-Null
}

function Stop-Worker {
    $config = Get-AppConfig
    $workers = Get-WorkerProcesses
    foreach ($worker in $workers) {
        Stop-Process -Id $worker.ProcessId -Force
    }

    $streamUrls = @($config.Streams)
    if ($streamUrls.Count -eq 0) {
        return
    }

    $downloadProcesses = Get-CimInstance Win32_Process |
        Where-Object {
            $processCommandLine = $_.CommandLine
            $_.Name -match "yt-dlp" -and
            ($streamUrls | Where-Object { -not [string]::IsNullOrWhiteSpace($processCommandLine) -and $processCommandLine.Contains($_) }).Count -gt 0
        }

    foreach ($process in @($downloadProcesses)) {
        Stop-Process -Id $process.ProcessId -Force
    }
}

function Test-StreamUrl {
    param([string]$Url)

    if ([string]::IsNullOrWhiteSpace($Url)) {
        return $false
    }

    $uri = $null
    if (-not [System.Uri]::TryCreate($Url.Trim(), [System.UriKind]::Absolute, [ref]$uri)) {
        return $false
    }

    return $uri.Scheme -in @("http", "https")
}

function Get-StatusMap {
    $config = Get-AppConfig
    $script:StatusPath = Join-Path $config.LogDirectory "status.json"
    $map = @{}

    if (Test-Path -LiteralPath $script:StatusPath) {
        try {
            $status = Get-Content -LiteralPath $script:StatusPath -Raw | ConvertFrom-Json
            foreach ($item in @($status.Streams)) {
                $map[[string]$item.Url] = $item
            }
        }
        catch {
            return @{}
        }
    }

    return $map
}

New-DirectoryIfMissing -Path $script:DefaultDownloadDirectory
New-DirectoryIfMissing -Path $script:DefaultLogDirectory
Save-AppConfig -Config (Get-AppConfig)

$form = New-Object System.Windows.Forms.Form
$form.Text = "Live Downloader"
$form.StartPosition = "CenterScreen"
$form.MinimumSize = New-Object System.Drawing.Size(880, 560)
$form.Size = New-Object System.Drawing.Size(980, 640)
$form.Font = New-Object System.Drawing.Font("Segoe UI", 10)
$form.BackColor = [System.Drawing.Color]::FromArgb(248, 249, 251)

$header = New-Object System.Windows.Forms.Label
$header.Text = "Live Downloader"
$header.Font = New-Object System.Drawing.Font("Segoe UI Semibold", 18)
$header.AutoSize = $true
$header.Location = New-Object System.Drawing.Point(18, 16)
$form.Controls.Add($header)

$statusLabel = New-Object System.Windows.Forms.Label
$statusLabel.AutoSize = $true
$statusLabel.Location = New-Object System.Drawing.Point(22, 58)
$form.Controls.Add($statusLabel)

$urlLabel = New-Object System.Windows.Forms.Label
$urlLabel.Text = "Stream URL"
$urlLabel.AutoSize = $true
$urlLabel.Location = New-Object System.Drawing.Point(22, 96)
$form.Controls.Add($urlLabel)

$urlTextBox = New-Object System.Windows.Forms.TextBox
$urlTextBox.Anchor = "Top,Left,Right"
$urlTextBox.Location = New-Object System.Drawing.Point(22, 122)
$urlTextBox.Size = New-Object System.Drawing.Size(700, 28)
$form.Controls.Add($urlTextBox)

$addButton = New-Object System.Windows.Forms.Button
$addButton.Anchor = "Top,Right"
$addButton.Text = "Add URL"
$addButton.Location = New-Object System.Drawing.Point(736, 120)
$addButton.Size = New-Object System.Drawing.Size(96, 32)
$form.Controls.Add($addButton)

$removeButton = New-Object System.Windows.Forms.Button
$removeButton.Anchor = "Top,Right"
$removeButton.Text = "Remove"
$removeButton.Location = New-Object System.Drawing.Point(842, 120)
$removeButton.Size = New-Object System.Drawing.Size(96, 32)
$form.Controls.Add($removeButton)

$grid = New-Object System.Windows.Forms.DataGridView
$grid.Anchor = "Top,Bottom,Left,Right"
$grid.Location = New-Object System.Drawing.Point(22, 168)
$grid.Size = New-Object System.Drawing.Size(916, 260)
$grid.AllowUserToAddRows = $false
$grid.AllowUserToDeleteRows = $false
$grid.MultiSelect = $false
$grid.SelectionMode = "FullRowSelect"
$grid.ReadOnly = $true
$grid.RowHeadersVisible = $false
$grid.AutoSizeColumnsMode = "Fill"
$grid.BackgroundColor = [System.Drawing.Color]::White
$grid.BorderStyle = "FixedSingle"
$grid.Columns.Add("Url", "URL") | Out-Null
$grid.Columns.Add("State", "State") | Out-Null
$grid.Columns.Add("ProcessId", "PID") | Out-Null
$grid.Columns.Add("LastProbeAt", "Last Probe") | Out-Null
$grid.Columns.Add("LastMessage", "Message") | Out-Null
$grid.Columns["Url"].FillWeight = 190
$grid.Columns["State"].FillWeight = 60
$grid.Columns["ProcessId"].FillWeight = 40
$grid.Columns["LastProbeAt"].FillWeight = 70
$grid.Columns["LastMessage"].FillWeight = 120
$form.Controls.Add($grid)

$startButton = New-Object System.Windows.Forms.Button
$startButton.Anchor = "Bottom,Left"
$startButton.Text = "Start Watching"
$startButton.Location = New-Object System.Drawing.Point(22, 446)
$startButton.Size = New-Object System.Drawing.Size(130, 36)
$form.Controls.Add($startButton)

$stopButton = New-Object System.Windows.Forms.Button
$stopButton.Anchor = "Bottom,Left"
$stopButton.Text = "Stop All"
$stopButton.Location = New-Object System.Drawing.Point(164, 446)
$stopButton.Size = New-Object System.Drawing.Size(96, 36)
$form.Controls.Add($stopButton)

$downloadsButton = New-Object System.Windows.Forms.Button
$downloadsButton.Anchor = "Bottom,Left"
$downloadsButton.Text = "Downloads"
$downloadsButton.Location = New-Object System.Drawing.Point(272, 446)
$downloadsButton.Size = New-Object System.Drawing.Size(112, 36)
$form.Controls.Add($downloadsButton)

$settingsGroup = New-Object System.Windows.Forms.GroupBox
$settingsGroup.Anchor = "Bottom,Left,Right"
$settingsGroup.Text = "Settings"
$settingsGroup.Location = New-Object System.Drawing.Point(22, 496)
$settingsGroup.Size = New-Object System.Drawing.Size(916, 88)
$form.Controls.Add($settingsGroup)

$ytDlpLabel = New-Object System.Windows.Forms.Label
$ytDlpLabel.Text = "yt-dlp"
$ytDlpLabel.AutoSize = $true
$ytDlpLabel.Location = New-Object System.Drawing.Point(14, 34)
$settingsGroup.Controls.Add($ytDlpLabel)

$ytDlpTextBox = New-Object System.Windows.Forms.TextBox
$ytDlpTextBox.Anchor = "Top,Left,Right"
$ytDlpTextBox.Location = New-Object System.Drawing.Point(68, 31)
$ytDlpTextBox.Size = New-Object System.Drawing.Size(460, 28)
$settingsGroup.Controls.Add($ytDlpTextBox)

$browseButton = New-Object System.Windows.Forms.Button
$browseButton.Anchor = "Top,Right"
$browseButton.Text = "Browse"
$browseButton.Location = New-Object System.Drawing.Point(540, 29)
$browseButton.Size = New-Object System.Drawing.Size(82, 32)
$settingsGroup.Controls.Add($browseButton)

$intervalLabel = New-Object System.Windows.Forms.Label
$intervalLabel.Anchor = "Top,Right"
$intervalLabel.Text = "Check every"
$intervalLabel.AutoSize = $true
$intervalLabel.Location = New-Object System.Drawing.Point(642, 34)
$settingsGroup.Controls.Add($intervalLabel)

$intervalInput = New-Object System.Windows.Forms.NumericUpDown
$intervalInput.Anchor = "Top,Right"
$intervalInput.Minimum = 30
$intervalInput.Maximum = 86400
$intervalInput.Increment = 30
$intervalInput.Location = New-Object System.Drawing.Point(732, 31)
$intervalInput.Size = New-Object System.Drawing.Size(78, 28)
$settingsGroup.Controls.Add($intervalInput)

$secondsLabel = New-Object System.Windows.Forms.Label
$secondsLabel.Anchor = "Top,Right"
$secondsLabel.Text = "sec"
$secondsLabel.AutoSize = $true
$secondsLabel.Location = New-Object System.Drawing.Point(816, 34)
$settingsGroup.Controls.Add($secondsLabel)

$saveSettingsButton = New-Object System.Windows.Forms.Button
$saveSettingsButton.Anchor = "Top,Right"
$saveSettingsButton.Text = "Save"
$saveSettingsButton.Location = New-Object System.Drawing.Point(852, 29)
$saveSettingsButton.Size = New-Object System.Drawing.Size(52, 32)
$settingsGroup.Controls.Add($saveSettingsButton)

function Refresh-Ui {
    $config = Get-AppConfig
    $statusMap = Get-StatusMap
    $running = (Get-WorkerProcesses).Count -gt 0

    $statusLabel.Text = if ($running) { "Status: watching streams" } else { "Status: stopped" }
    if (-not $ytDlpTextBox.Focused) {
        $ytDlpTextBox.Text = $config.YtDlpPath
    }
    if (-not $intervalInput.Focused) {
        $intervalInput.Value = [decimal]$config.CheckIntervalSeconds
    }

    $selectedUrl = $null
    if ($grid.SelectedRows.Count -gt 0) {
        $selectedUrl = [string]$grid.SelectedRows[0].Cells["Url"].Value
    }

    $grid.Rows.Clear()
    foreach ($url in @($config.Streams)) {
        $state = "Watching"
        $pid = ""
        $lastProbe = ""
        $message = "Waiting for worker"

        if ($statusMap.ContainsKey($url)) {
            $item = $statusMap[$url]
            $state = [string]$item.State
            $pid = if ($null -ne $item.ProcessId) { [string]$item.ProcessId } else { "" }
            $lastProbe = if ($null -ne $item.LastProbeAt) { [string]$item.LastProbeAt } else { "" }
            $message = [string]$item.LastMessage
        }

        $index = $grid.Rows.Add($url, $state, $pid, $lastProbe, $message)
        if ($url -eq $selectedUrl) {
            $grid.Rows[$index].Selected = $true
        }
    }
}

$addButton.Add_Click({
    $url = $urlTextBox.Text.Trim()
    if (-not (Test-StreamUrl -Url $url)) {
        [System.Windows.Forms.MessageBox]::Show("Enter a valid http or https stream URL.", "Live Downloader", "OK", "Warning") | Out-Null
        return
    }

    $config = Get-AppConfig
    if (@($config.Streams) -notcontains $url) {
        $config.Streams = @($config.Streams) + $url
        Save-AppConfig -Config $config
    }

    $urlTextBox.Clear()
    Refresh-Ui
})

$removeButton.Add_Click({
    if ($grid.SelectedRows.Count -eq 0) {
        return
    }

    $url = [string]$grid.SelectedRows[0].Cells["Url"].Value
    $config = Get-AppConfig
    $config.Streams = @($config.Streams | Where-Object { $_ -ne $url })
    Save-AppConfig -Config $config
    Refresh-Ui
})

$startButton.Add_Click({
    $config = Get-AppConfig
    if (-not (Test-Path -LiteralPath $config.YtDlpPath)) {
        [System.Windows.Forms.MessageBox]::Show("yt-dlp was not found at:`r`n$($config.YtDlpPath)", "Live Downloader", "OK", "Warning") | Out-Null
        return
    }

    Start-Worker
    Start-Sleep -Milliseconds 300
    Refresh-Ui
})

$stopButton.Add_Click({
    Stop-Worker
    Start-Sleep -Milliseconds 300
    Refresh-Ui
})

$downloadsButton.Add_Click({
    $config = Get-AppConfig
    New-DirectoryIfMissing -Path $config.DownloadDirectory
    Start-Process -FilePath "explorer.exe" -ArgumentList $config.DownloadDirectory
})

$browseButton.Add_Click({
    $dialog = New-Object System.Windows.Forms.OpenFileDialog
    $dialog.Filter = "Executable files (*.exe)|*.exe|All files (*.*)|*.*"
    $dialog.FileName = "yt-dlp.exe"
    if ($dialog.ShowDialog($form) -eq "OK") {
        $ytDlpTextBox.Text = $dialog.FileName
    }
})

$saveSettingsButton.Add_Click({
    $config = Get-AppConfig
    $config.YtDlpPath = $ytDlpTextBox.Text.Trim()
    $config.CheckIntervalSeconds = [int]$intervalInput.Value
    Save-AppConfig -Config $config
    Refresh-Ui
})

$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 5000
$timer.Add_Tick({ Refresh-Ui })
$timer.Start()

$form.Add_Shown({ Refresh-Ui })
$form.Add_FormClosed({ $timer.Stop(); $timer.Dispose() })

[System.Windows.Forms.Application]::EnableVisualStyles()
[System.Windows.Forms.Application]::Run($form)

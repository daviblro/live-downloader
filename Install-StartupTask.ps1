param(
    [string]$TaskName = "LiveDownloader"
)

$scriptPath = Join-Path $PSScriptRoot "LiveDownloader.ps1"
$configPath = Join-Path $PSScriptRoot "config.json"
$actionArgs = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$scriptPath`" -ConfigPath `"$configPath`""
$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument $actionArgs -WorkingDirectory $PSScriptRoot
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action $action `
    -Trigger $trigger `
    -Principal $principal `
    -Settings $settings `
    -Description "Watches configured live stream URLs and downloads them when live." `
    -Force | Out-Null

Write-Host "Installed scheduled task '$TaskName'. It will run at logon."
Write-Host "Start it now with: Start-ScheduledTask -TaskName '$TaskName'"

param(
    [string]$TaskName = "SoucarlosdanielLiveDownloader"
)

$scriptPath = Join-Path $PSScriptRoot "LiveDownloader.ps1"
$actionArgs = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$scriptPath`""
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
    -Description "Checks Twitch every 5 minutes and downloads soucarlosdaniel when live." `
    -Force | Out-Null

Write-Host "Installed scheduled task '$TaskName'. It will run at logon."
Write-Host "Start it now with: Start-ScheduledTask -TaskName '$TaskName'"

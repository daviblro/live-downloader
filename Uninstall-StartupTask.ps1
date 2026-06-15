param(
    [string]$TaskName = "SoucarlosdanielLiveDownloader"
)

Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
Write-Host "Removed scheduled task '$TaskName'."

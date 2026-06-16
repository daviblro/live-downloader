param(
    [string]$TaskName = "LiveDownloader"
)

Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
Write-Host "Removed scheduled task '$TaskName'."

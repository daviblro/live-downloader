# Twitch Live Downloader

Small Windows PowerShell background app that checks every 5 minutes for:

```text
https://www.twitch.tv/soucarlosdaniel
```

When the stream is live, it starts:

```powershell
C:\ffmpeg\yt-dlp_x86.exe --no-cache https://www.twitch.tv/soucarlosdaniel
```

Downloads are saved in `downloads/`. Logs are saved in `logs/`.

## Requirements

- Windows PowerShell
- `C:\ffmpeg\yt-dlp_x86.exe`

## Run Once In Background

```powershell
.\Start-LiveDownloader.ps1
```

## Stop

```powershell
.\Stop-LiveDownloader.ps1
```

## Start Automatically When You Log In

Install the scheduled task:

```powershell
.\Install-StartupTask.ps1
```

Start it immediately without waiting for the next login:

```powershell
Start-ScheduledTask -TaskName "SoucarlosdanielLiveDownloader"
```

Remove the scheduled task:

```powershell
.\Uninstall-StartupTask.ps1
```

## Configuration

You can run `LiveDownloader.ps1` directly with custom values:

```powershell
.\LiveDownloader.ps1 `
  -StreamUrl "https://www.twitch.tv/soucarlosdaniel" `
  -YtDlpPath "C:\ffmpeg\yt-dlp_x86.exe" `
  -CheckIntervalSeconds 300
```

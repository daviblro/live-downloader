# Live Downloader

Simple Windows PowerShell app for watching one or more live stream URLs and downloading each stream when it goes live.

The app stores its settings in `config.json`, downloads videos into `downloads/`, and writes logs/status into `logs/`.

## Requirements

- Windows PowerShell
- `yt-dlp` executable, defaulting to `C:\ffmpeg\yt-dlp_x86.exe`

## Open The Windows App

```powershell
.\Start-LiveDownloader.ps1
```

From the app you can:

- add more than one stream URL
- remove URLs you no longer want to watch
- start or stop watching
- download multiple live streams at the same time
- choose the `yt-dlp` executable path
- change how often streams are checked
- open the downloads folder

## Background Worker

The visible app starts `LiveDownloader.ps1` as a hidden worker when you click **Start Watching**.

You can also run the worker directly:

```powershell
.\LiveDownloader.ps1
```

Or with custom values the first time it creates `config.json`:

```powershell
.\LiveDownloader.ps1 `
  -StreamUrl "https://www.twitch.tv/example1","https://www.twitch.tv/example2" `
  -YtDlpPath "C:\ffmpeg\yt-dlp_x86.exe" `
  -CheckIntervalSeconds 300
```

The worker reloads `config.json` on every check cycle, so URLs added in the app are picked up automatically.

## Stop Everything

```powershell
.\Stop-LiveDownloader.ps1
```

This stops the hidden worker and any active `yt-dlp` downloads for the configured stream URLs.

## Start Automatically When You Log In

Install the scheduled task:

```powershell
.\Install-StartupTask.ps1
```

Start it immediately without waiting for the next login:

```powershell
Start-ScheduledTask -TaskName "LiveDownloader"
```

Remove the scheduled task:

```powershell
.\Uninstall-StartupTask.ps1
```

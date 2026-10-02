# Live Downloader

Live Downloader is a Windows desktop app for monitoring authorised live-stream
URLs and recording them locally when they become available. It runs quietly in
the system tray, supports automatic startup, and keeps the watch list and
recording history on the device.

The installer includes `yt-dlp`, FFmpeg, and FFprobe. No separate media-tool
installation or PATH setup is required.

## Install

1. Open the [GitHub Releases page](https://github.com/daviblro/live-downloader/releases).
2. Download the latest `*_x64-setup.exe` installer.
3. Run the installer, then start **Live Downloader** from the Start menu.
4. Select **Add stream**, give the source a name, and enter an authorised HTTP or
   HTTPS stream URL.

The installer is per-user and normally does not request administrator access. If
Microsoft WebView2 is not already installed, Windows downloads it during setup.

## Using the app

- Keep sources in the **Watch list** and use **Pause all** when you want to stop
  monitoring.
- Configure the download folder, monitoring limits, notifications, appearance,
  and **Start with Windows** under **Settings**.
- Optionally select **Connect Twitch** under **Settings** to use Twitch channel
  profile pictures. Monitoring and recording continue to work without a Twitch
  connection.
- Recordings use `channel - mm-dd-yyyy hh-mm-ss.ext` in English and
  `channel - dd-mm-yyyy hh-mm-ss.ext` in Portuguese, with Windows-safe separators.
- **Stop recording** ends the current broadcast's recording; the source is not
  recorded again until it goes offline or you select **Check now**. If a
  recording drops unexpectedly, the source is re-checked within seconds so the
  recording resumes in a new file.
- A stopped or interrupted recording is kept as a playable `.ts` file instead of
  an unfinished `.part` file. Recording does not start when less than 1 GB is
  free on the recording drive.
- Closing the window keeps the recorder available in the system tray; choose
  **Exit Live Downloader** from the tray menu to stop it completely.
- The app checks GitHub Releases at launch and shows a non-intrusive notice when
  a newer installer is available. Updates are installed by downloading the new
  installer from GitHub Releases.

Only record streams you are authorised to capture. See
[third-party notices](THIRD_PARTY_NOTICES.md) for bundled-component licensing.

## Twitch integration for builds

Twitch connection uses the public Device Code Flow and does not embed a client
secret. Register a public application in the Twitch Developer Console, then set
`TWITCH_CLIENT_ID` while compiling the Tauri application:

```powershell
$env:TWITCH_CLIENT_ID = "your-public-client-id"
pnpm exec tauri build
```

Release and validation workflows read the same value from a repository-level
GitHub Actions variable named `TWITCH_CLIENT_ID`, falling back to a secret with
the same name. If both are absent, the app builds normally and shows the optional
Twitch connection as unavailable.

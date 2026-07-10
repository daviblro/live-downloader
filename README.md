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
- Closing the window keeps the recorder available in the system tray; choose
  **Exit Live Downloader** from the tray menu to stop it completely.
- The app checks GitHub Releases at launch and shows a non-intrusive notice when
  a newer installer is available. Updates are installed by downloading the new
  installer from GitHub Releases.

Only record streams you are authorised to capture. See
[third-party notices](THIRD_PARTY_NOTICES.md) for bundled-component licensing.

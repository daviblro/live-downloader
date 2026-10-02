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
  a newer version is available. Select **Update now** to download, verify, and
  install it in place; Live Downloader restarts afterwards. Active recordings are
  stopped first and the footage recorded so far is kept. Releases published
  without updater signatures fall back to a link to the installer on GitHub
  Releases.

Only record streams you are authorised to capture. See
[third-party notices](THIRD_PARTY_NOTICES.md) for bundled-component licensing.

## In-app updates for releases

The in-app updater uses only GitHub Releases. Each signed release publishes the
installer, its `.sig` signature, and a `latest.json` manifest; the app reads
`releases/latest/download/latest.json`, verifies the installer signature with a
public key compiled into the app, and refuses an installer signed for a
different version.

To enable it, generate an updater key pair once (keep the private key and its
password backed up: without them, existing installs can no longer update):

```powershell
pnpm tauri signer generate -w "$HOME\.tauri\live-downloader.key"
```

Then add these to the repository under **Settings → Secrets and variables →
Actions**:

| Name                                 | Kind     | Value                                 |
| ------------------------------------ | -------- | ------------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY`          | Secret   | Contents of `live-downloader.key`     |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Secret   | The password chosen during generation |
| `LIVE_DOWNLOADER_UPDATER_PUBKEY`     | Variable | Contents of `live-downloader.key.pub` |

The release workflow signs the installer and publishes the manifest when these
are configured, and fails if only one of the key halves is present. Without
them, releases are published as before and the app shows the manual update
notice. To build a signed installer locally, set the same three environment
variables, run `node scripts/write-updater-config.mjs src-tauri/target/updater.conf.json`,
then `pnpm exec tauri build --bundles nsis --config src-tauri/target/updater.conf.json`.

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

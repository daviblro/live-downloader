# Third-party notices

## yt-dlp

The Windows installer packages `yt-dlp` version `2026.06.09` as a managed
recording sidecar. yt-dlp is distributed under the Unlicense. Its source,
license, and release information are available at
<https://github.com/yt-dlp/yt-dlp>.

The bundled executable is the existing user-configured Windows executable from
`C:\ffmpeg\yt-dlp_x86.exe`, renamed with Tauri's x64 target-triple convention.
Windows x64 can execute this compatible x86 binary. It must be refreshed through
the release workflow when a supported yt-dlp update is adopted.

## FFmpeg

FFmpeg is not currently packaged because no redistributable build was present in
the legacy installation. Live Downloader detects and uses yt-dlp without it for
supported direct streams. Before enabling workflows that require post-processing
or stream merging, select an FFmpeg distribution, include its matching license
notices, and add it to the sidecar release manifest.

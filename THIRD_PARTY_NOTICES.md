# Third-party notices

## yt-dlp

The Windows installer packages `yt-dlp` version `2026.06.09` as a managed
recording sidecar. yt-dlp is distributed under the Unlicense. Its source,
license, and release information are available at
<https://github.com/yt-dlp/yt-dlp>.

The build downloads the pinned official `yt-dlp.exe` release asset and verifies
its SHA-256 digest from `tools/sidecars.json` before packaging it with Tauri's
x64 target-triple filename.

## FFmpeg

The Windows installer packages `ffmpeg.exe` and `ffprobe.exe` from the Gyan.dev
FFmpeg 64-bit static essentials build:

- Version: `9.0-essentials_build`
- Distribution source: <https://github.com/GyanD/codexffmpeg/releases/tag/9.0>
- License: GNU General Public License version 3

The exact GPL text and the build README/source reference are bundled in the
installed application resources under `resources/third-party/ffmpeg`. The
release archive is pinned and checksum-verified through `tools/sidecars.json`.
executables remain separate sidecar processes; this notice is not legal advice.
Review the GPL obligations for the intended distribution model before publishing
the installer.

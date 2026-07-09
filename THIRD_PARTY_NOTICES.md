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

The Windows installer packages `ffmpeg.exe` and `ffprobe.exe` from the Gyan.dev
FFmpeg 64-bit static essentials build:

- Version: `2026-06-08-git-6028720d70-essentials_build-www.gyan.dev`
- Upstream FFmpeg source commit: `6028720d70`
- Distribution source: <https://www.gyan.dev/ffmpeg/builds/>
- License: GNU General Public License version 3

The exact GPL text and the build README/source reference are bundled in the
installed application resources under `resources/third-party/ffmpeg`. The
executables remain separate sidecar processes; this notice is not legal advice.
Review the GPL obligations for the intended distribution model before publishing
the installer.

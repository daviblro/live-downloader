# Releasing Live Downloader for Windows

## Version and tag

Keep the version aligned in `package.json`, `src-tauri/Cargo.toml`, and
`src-tauri/tauri.conf.json`. Create an annotated Git tag named `v<version>` from
`main` (for example, `v1.0.0`) and push it to GitHub. The **Publish Windows
release** workflow validates the version, builds the NSIS installer, and uploads
it to the matching GitHub Release.

The app does not use Tauri's in-place updater. At launch it reads the public
GitHub Releases API and, if a newer stable release is available, offers a button
to open that release in the user's browser.

## Local build

Prerequisites:

- Windows 10/11 x64 build host with the Rust MSVC toolchain and Node.js.
- Managed executables at `src-tauri/binaries/` for yt-dlp, FFmpeg, and FFprobe.
- A review of [`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) whenever a
  bundled sidecar changes.

Build and inspect the per-user NSIS installer:

```powershell
pnpm install --frozen-lockfile
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm exec tauri build --bundles nsis
```

The installer is written to:

```text
src-tauri/target/release/bundle/nsis/
```

It keeps the download library intact on uninstall. The installer uses WebView2's
download bootstrapper, so an unprovisioned Windows computer needs internet access
once during setup.

## GitHub Actions

- **Validate** runs on pull requests to `main` and every push to `main`. It type
  checks the frontend, runs Rust tests, builds the NSIS installer, and retains the
  installer as a short-lived workflow artifact.
- **Publish Windows release** runs only for `v*` tags. It builds the installer on
  a clean Windows runner and attaches it to the GitHub Release. The workflow uses
  GitHub's built-in token and requires `contents: write` permission.

## Signing and licensing

Before broad distribution, sign the application and setup executable with a
Windows code-signing certificate and timestamp the signatures. Store signing
material only in GitHub Actions secrets; never commit it.

The installer packages the Gyan.dev 64-bit static essentials build of FFmpeg and
FFprobe and passes its deployed directory to yt-dlp through `--ffmpeg-location`.
Update the target-triple binaries, GPL/source resource files, and
[`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) together. Do not package
`ffplay`, the FFmpeg documentation tree, presets, or unrelated legacy binaries.

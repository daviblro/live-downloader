# Releasing Live Downloader for Windows

## Version and tag

Keep the version aligned in `package.json` and `src-tauri/Cargo.toml` (the Tauri
configuration reads `package.json` directly). Create an annotated Git tag named `v<version>` from
`main` (for example, `v1.0.0`) and push it to GitHub. The **Publish Windows
release** workflow validates the version, builds the NSIS installer, and uploads
it to the matching GitHub Release.

Set both version fields with one command before creating the release tag:

```powershell
pnpm version:bump 1.0.3
```

The command accepts a semantic version (including prerelease and build metadata)
and reports each updated file. It does not create a commit or Git tag.

## In-app updates

Releases built with the updater signing key publish three assets: the NSIS
installer, its `.sig` signature, and `latest.json`. Installed apps read
`releases/latest/download/latest.json`, verify the installer against the public
key embedded at build time, and install it in place. Signatures are bound to the
release version, so `latest.json` cannot point a version number at another
release's installer.

Signing requires the `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets and the
`LIVE_DOWNLOADER_UPDATER_PUBKEY` variable (see the README for setup). Without
them the workflow publishes only the installer and the app falls back to a link
to the GitHub release. Never rotate or discard the key casually: installs built
with one public key accept updates signed only by its private key.

After publishing, confirm the release lists all three assets and that
`latest.json` names the new version.

## Local build

Prerequisites:

- Windows 10/11 x64 build host with the Rust MSVC toolchain and Node.js.
- Internet access to fetch the pinned, checksum-verified sidecars.
- A review of [`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) whenever a
  bundled sidecar changes.

Build and inspect the per-user NSIS installer:

```powershell
pnpm install --frozen-lockfile
pnpm version:check
pnpm sidecars:fetch
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
  a clean Windows runner, signs updater artifacts when the key is configured, and
  attaches the installer, signature, and `latest.json` to the GitHub Release. The workflow uses
  GitHub's built-in token and requires `contents: write` permission.

## Signing and licensing

Before broad distribution, sign the application and setup executable with a
Windows code-signing certificate and timestamp the signatures. Store signing
material only in GitHub Actions secrets; never commit it.

The installer packages the Gyan.dev 64-bit static essentials build of FFmpeg and
FFprobe and passes its deployed directory to yt-dlp through `--ffmpeg-location`.
Update `tools/sidecars.json`, GPL/source resource files, and
[`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) together. Do not package
`ffplay`, the FFmpeg documentation tree, presets, or unrelated legacy binaries.

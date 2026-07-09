# Releasing Live Downloader for Windows

## Prerequisites

- Windows 10/11 x64 build host with the Rust MSVC toolchain and Node.js installed.
- The managed sidecar at
  `src-tauri/binaries/yt-dlp-x86_64-pc-windows-msvc.exe`.
- Review [`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) whenever the
  sidecar changes. Do not silently replace it with an unverified binary.

## Build the installer

```powershell
pnpm install
pnpm build
pnpm exec tauri build --bundles nsis
```

The per-user NSIS installer is written under:

```text
src-tauri/target/release/bundle/nsis/
```

It installs beneath the current user's `%LOCALAPPDATA%` directory, so routine
installs do not require elevation. The bundled installer hook deliberately leaves
the user's download library untouched when the app is removed.

## Signing

Before distributing outside a private test group, sign the application and setup
executable with a Windows code-signing certificate and timestamp every signature.
Set the certificate thumbprint and timestamp URL in a private Tauri config overlay
or CI secret; never commit either credential to this repository.

## Application updates

The updater plugin is intentionally inactive in `tauri.conf.json` until there is a
controlled HTTPS release endpoint and an offline-held Tauri signing key. To enable
it for a release channel:

1. Generate a Tauri updater signing key and store it in the CI secret manager.
2. Publish signed NSIS artifacts plus the channel JSON manifest to an HTTPS
   endpoint owned by the product.
3. Add the public key and `endpoints` to a private production config overlay, then
   set `plugins.updater.active` to `true` for the release build only.
4. Test an update from the preceding installer on a clean Windows VM before
   promoting the release.

This keeps an unconfigured updater from advertising or accepting unknown releases.

## FFmpeg follow-up

The current installer packages yt-dlp. Select and audit a redistributable FFmpeg
build before adding it to `bundle.externalBin`; then update the third-party notice,
sidecar manifest, and clean-VM tests together.

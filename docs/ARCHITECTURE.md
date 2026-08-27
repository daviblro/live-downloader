# Architecture

Live Downloader is one Tauri application. React owns presentation and user interaction; Rust owns persistence, monitoring, sidecars, platform integration, and validation.

## Frontend

`src/app` composes navigation and application state. Feature folders contain pages and feature-specific components. `src/shared` contains contracts, translations, reusable components, date helpers, and the typed Tauri client. Shared code does not import app or feature modules.

## IPC

The client in `src/shared/tauri/client.ts` invokes thin commands under `src-tauri/src/commands`. Commands validate input and delegate to monitoring, persistence, or platform modules. Engine changes use the `engine://changed` event; the frontend responds through one authoritative bootstrap refresh path.

## Rust modules

- `app`: state and tray/window behavior.
- `commands`: Tauri IPC boundaries.
- `domain`: serialized settings, targets, recordings, and states.
- `monitoring`: scheduler/runtime state, probes, recording processes, and output reconciliation.
- `persistence`: the SQLite connection, versioned migrations, targets, jobs, and settings.
- `platform`: disk usage and Windows shell integration.

## SQLite lifecycle

One `Database` instance owns the SQLite connection. `PRAGMA user_version` migrations run transactionally during startup. Watch targets use soft deletion so recording history remains linked; re-adding a removed URL restores its existing target row.

## Recording lifecycle

The engine reserves probes and recording slots while holding its runtime lock. Live targets beyond capacity enter a FIFO queue and are probed again when a slot opens. Recording completion releases the slot and drains the queue. Jobs left in `Recording` after an abnormal shutdown are marked `Interrupted` before scheduling starts.

## Sidecars and releases

`tools/sidecars.json` pins yt-dlp and FFmpeg downloads and SHA-256 checksums. `pnpm sidecars:fetch` verifies artifacts before extracting only the required executables into the ignored Tauri binaries directory. CI fetches sidecars before building NSIS installers. Release tags must match the versions in `package.json` and `src-tauri/Cargo.toml`.

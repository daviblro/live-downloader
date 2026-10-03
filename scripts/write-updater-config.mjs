// Writes a Tauri config overlay that enables signed updater artifacts using the
// public key from LIVE_DOWNLOADER_UPDATER_PUBKEY. The bundler verifies each
// signature against this key, and the app embeds it to verify downloads.
//
// Usage: node scripts/write-updater-config.mjs <output-file>
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const [outputFile] = process.argv.slice(2);
if (!outputFile) throw new Error("Usage: node scripts/write-updater-config.mjs <output-file>");

const pubkey = process.env.LIVE_DOWNLOADER_UPDATER_PUBKEY?.trim();
if (!pubkey) throw new Error("Set LIVE_DOWNLOADER_UPDATER_PUBKEY to the updater public key.");

const decoded = Buffer.from(pubkey, "base64").toString("utf8");
if (!decoded.startsWith("untrusted comment:")) {
  throw new Error(
    "LIVE_DOWNLOADER_UPDATER_PUBKEY must be the full contents of the generated .key.pub file.",
  );
}

await mkdir(path.dirname(outputFile), { recursive: true });
await writeFile(
  outputFile,
  `${JSON.stringify(
    { bundle: { createUpdaterArtifacts: true }, plugins: { updater: { pubkey } } },
    null,
    2,
  )}\n`,
);
console.log(`Wrote updater build configuration to ${outputFile}.`);

// Collects the signed NSIS installer and writes the `latest.json` manifest read
// by the in-app updater from the latest GitHub release.
//
// Usage: node scripts/write-updater-manifest.mjs <bundle-directory> <output-directory>
import { copyFile, mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const [bundleDirectory, outputDirectory] = process.argv.slice(2);
if (!bundleDirectory || !outputDirectory) {
  throw new Error(
    "Usage: node scripts/write-updater-manifest.mjs <bundle-directory> <output-directory>",
  );
}

const repository = process.env.GITHUB_REPOSITORY ?? "daviblro/live-downloader";
const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
const version = packageJson.version;
const tag = `v${version}`;

const installers = (await readdir(bundleDirectory)).filter((name) => name.endsWith("-setup.exe"));
if (installers.length !== 1) {
  throw new Error(`Expected exactly one NSIS installer, found ${installers.length}.`);
}
const installer = installers[0];
const signatureFile = path.join(bundleDirectory, `${installer}.sig`);
const signature = (await readFile(signatureFile, "utf8").catch(() => "")).trim();
if (!signature) {
  throw new Error(
    `Missing updater signature ${signatureFile}. Build with TAURI_SIGNING_PRIVATE_KEY and createUpdaterArtifacts enabled.`,
  );
}

// The updater requires the signed version (requireSignedVersion) to match the
// manifest, so refuse to publish a signature bound to another version.
const trustedComment = Buffer.from(signature, "base64")
  .toString("utf8")
  .split(/\r?\n/)
  .find((line) => line.startsWith("trusted comment:"));
const signedVersion = trustedComment?.match(/\bversion:(\S+)/)?.[1];
if (signedVersion !== version) {
  throw new Error(
    `The installer signature is bound to version ${signedVersion ?? "(none)"}, expected ${version}.`,
  );
}

// GitHub replaces spaces in asset names with dots; publish under that name so
// the manifest URL matches the uploaded asset exactly.
const assetName = installer.replaceAll(" ", ".");
await mkdir(outputDirectory, { recursive: true });
await copyFile(path.join(bundleDirectory, installer), path.join(outputDirectory, assetName));
await writeFile(path.join(outputDirectory, `${assetName}.sig`), `${signature}\n`);

const manifest = {
  version,
  notes: `See https://github.com/${repository}/releases/tag/${tag}`,
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature,
      url: `https://github.com/${repository}/releases/download/${tag}/${encodeURIComponent(assetName)}`,
    },
  },
};
await writeFile(
  path.join(outputDirectory, "latest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
console.log(`Wrote updater manifest for ${assetName} (${tag}).`);

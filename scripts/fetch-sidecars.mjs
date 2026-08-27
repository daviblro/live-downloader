import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, readdir, rm, writeFile, copyFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { execFile } from "node:child_process";

const run = promisify(execFile);
const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(
  await readFile(path.join(projectRoot, "tools", "sidecars.json"), "utf8"),
);
const outputDirectory = path.join(projectRoot, "src-tauri", "binaries");
const temporaryDirectory = await mkdtemp(path.join(tmpdir(), "live-downloader-sidecars-"));

async function download(url, destination) {
  const response = await fetch(url, { redirect: "follow" });
  if (!response.ok) throw new Error(`Download failed (${response.status}) for ${url}`);
  await writeFile(destination, Buffer.from(await response.arrayBuffer()));
}

async function verify(file, expected) {
  const digest = createHash("sha256")
    .update(await readFile(file))
    .digest("hex");
  if (digest !== expected.toLowerCase()) {
    throw new Error(
      `SHA-256 mismatch for ${path.basename(file)}: expected ${expected}, received ${digest}`,
    );
  }
}

async function findExactlyOne(directory, filename) {
  const matches = [];
  async function walk(current) {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const candidate = path.join(current, entry.name);
      if (entry.isDirectory()) await walk(candidate);
      else if (entry.name.toLowerCase() === filename.toLowerCase()) matches.push(candidate);
    }
  }
  await walk(directory);
  if (matches.length !== 1)
    throw new Error(`Expected exactly one ${filename}, found ${matches.length}`);
  return matches[0];
}

try {
  await mkdir(outputDirectory, { recursive: true });

  const ytDlpDownload = path.join(temporaryDirectory, "yt-dlp.exe");
  await download(manifest.ytDlp.url, ytDlpDownload);
  await verify(ytDlpDownload, manifest.ytDlp.sha256);

  const ffmpegArchive = path.join(temporaryDirectory, "ffmpeg.zip");
  await download(manifest.ffmpeg.url, ffmpegArchive);
  await verify(ffmpegArchive, manifest.ffmpeg.sha256);

  const archiveList = (await run("tar", ["-tf", ffmpegArchive])).stdout.split(/\r?\n/);
  const executableEntries = ["ffmpeg.exe", "ffprobe.exe"].map((filename) => {
    const matches = archiveList.filter((entry) => entry.toLowerCase().endsWith(`/bin/${filename}`));
    if (matches.length !== 1)
      throw new Error(`Expected exactly one ${filename} in the archive, found ${matches.length}`);
    return matches[0];
  });
  const extractionDirectory = path.join(temporaryDirectory, "ffmpeg");
  await mkdir(extractionDirectory);
  await run("tar", ["-xf", ffmpegArchive, "-C", extractionDirectory, ...executableEntries]);

  const ffmpegExecutable = await findExactlyOne(extractionDirectory, "ffmpeg.exe");
  const ffprobeExecutable = await findExactlyOne(extractionDirectory, "ffprobe.exe");
  const suffix = manifest.targetTriple;
  await Promise.all([
    copyFile(ytDlpDownload, path.join(outputDirectory, `yt-dlp-${suffix}.exe`)),
    copyFile(ffmpegExecutable, path.join(outputDirectory, `ffmpeg-${suffix}.exe`)),
    copyFile(ffprobeExecutable, path.join(outputDirectory, `ffprobe-${suffix}.exe`)),
  ]);
  console.log(
    `Fetched verified yt-dlp ${manifest.ytDlp.version} and FFmpeg ${manifest.ffmpeg.version}.`,
  );
} finally {
  await rm(temporaryDirectory, { recursive: true, force: true });
}

import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const version = process.argv[2];
const semverPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

if (!version || !semverPattern.test(version)) {
  console.error("Usage: pnpm version:bump <semver>");
  console.error("Example: pnpm version:bump 1.0.3");
  process.exitCode = 1;
} else {
  const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const versionFiles = [
    {
      path: "package.json",
      pattern: /^(  "version": ")[^"]+(",)$/m,
      replacement: `$1${version}$2`,
    },
    {
      path: "src-tauri/Cargo.toml",
      pattern: /^(version = ")[^"]+("$)/m,
      replacement: `$1${version}$2`,
    },
    {
      path: "src-tauri/tauri.conf.json",
      pattern: /^(  "version": ")[^"]+(",)$/m,
      replacement: `$1${version}$2`,
    },
  ];

  const updates = await Promise.all(
    versionFiles.map(async ({ path: relativePath, pattern, replacement }) => {
      const absolutePath = path.join(projectRoot, relativePath);
      const contents = await readFile(absolutePath, "utf8");

      if (!pattern.test(contents)) {
        throw new Error(`Could not find the version field in ${relativePath}.`);
      }

      return {
        absolutePath,
        relativePath,
        contents: contents.replace(pattern, replacement),
      };
    }),
  );

  await Promise.all(
    updates.map(({ absolutePath, contents }) => writeFile(absolutePath, contents, "utf8")),
  );

  console.log(`Updated project version to ${version}:`);
  for (const { relativePath } of updates) {
    console.log(`- ${relativePath}`);
  }
}

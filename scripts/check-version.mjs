import { readFile } from "node:fs/promises";

const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
const cargoToml = await readFile(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");
const cargoVersion = cargoToml.match(/^version = "([^"]+)"$/m)?.[1];

if (!cargoVersion) {
  throw new Error("Could not read the package version from src-tauri/Cargo.toml.");
}

if (packageJson.version !== cargoVersion) {
  throw new Error(
    `Version mismatch: package.json=${packageJson.version}, src-tauri/Cargo.toml=${cargoVersion}`,
  );
}

console.log(`Version ${packageJson.version} is consistent.`);

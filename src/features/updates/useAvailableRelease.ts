import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { api, isDesktop } from "../../shared/tauri/client";
import type { AvailableRelease } from "./components/ReleaseNotice";
import { isNewerRelease, type GitHubRelease } from "./release";

export const releasesPageUrl = "https://github.com/daviblro/live-downloader/releases";
const releasesApiUrl = "https://api.github.com/repos/daviblro/live-downloader/releases/latest";

async function findInstallableRelease(): Promise<AvailableRelease | null | undefined> {
  try {
    const check = await api.checkForUpdate();
    if (!check.configured) return undefined;
    if (!check.update) return null;
    return {
      version: check.update.version,
      url: `${releasesPageUrl}/tag/v${check.update.version.replace(/^v/i, "")}`,
      installable: true,
    };
  } catch {
    // Releases published without updater metadata fall back to the notice.
    return undefined;
  }
}

async function findPublishedRelease(): Promise<AvailableRelease | null> {
  const [installedVersion, response] = await Promise.all([
    getVersion(),
    fetch(releasesApiUrl, { headers: { Accept: "application/vnd.github+json" } }),
  ]);
  if (!response.ok) return null;
  const release = (await response.json()) as GitHubRelease;
  if (
    release.draft ||
    release.prerelease ||
    typeof release.tag_name !== "string" ||
    typeof release.html_url !== "string" ||
    !isNewerRelease(release.tag_name, installedVersion)
  )
    return null;
  return { version: release.tag_name, url: release.html_url, installable: false };
}

/** Looks for a newer release once at launch, preferring the signed in-app updater. */
export function useAvailableRelease() {
  const [release, setRelease] = useState<AvailableRelease | null>(null);
  useEffect(() => {
    if (!isDesktop) return;
    let cancelled = false;
    void (async () => {
      const installable = await findInstallableRelease();
      const found = installable === undefined ? await findPublishedRelease() : installable;
      if (!cancelled && found) setRelease(found);
    })().catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);
  return [release, setRelease] as const;
}

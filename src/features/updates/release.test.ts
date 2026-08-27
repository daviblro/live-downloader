import { describe, expect, it } from "vitest";
import { displayReleaseVersion, isNewerRelease } from "./release";

describe("release version helpers", () => {
  it("detects newer semantic versions", () => {
    expect(isNewerRelease("v1.1.0", "1.0.5")).toBe(true);
    expect(isNewerRelease("1.0.5", "1.0.5")).toBe(false);
    expect(isNewerRelease("1.0.4", "1.0.5")).toBe(false);
  });

  it("rejects malformed release tags", () => {
    expect(isNewerRelease("latest", "1.0.5")).toBe(false);
  });

  it("removes a leading v for display", () => {
    expect(displayReleaseVersion("v2.3.4")).toBe("2.3.4");
  });
});

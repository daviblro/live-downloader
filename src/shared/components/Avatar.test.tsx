import { describe, expect, it } from "vitest";
import { avatarColor } from "./Avatar";

describe("avatarColor", () => {
  it("returns the same color for the same channel identity", () => {
    expect(avatarColor("target-42")).toBe(avatarColor("target-42"));
  });

  it("ignores letter casing when the channel name is used as the identity", () => {
    expect(avatarColor("TwitchDev")).toBe(avatarColor("twitchdev"));
  });
});

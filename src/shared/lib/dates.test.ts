import { describe, expect, it } from "vitest";
import { formatDateTime, formatTime } from "./dates";

describe("date formatting", () => {
  const value = new Date(2024, 0, 2, 15, 4);

  it("uses the configured locale", () => {
    expect(formatDateTime(value, "en")).toBe(
      new Intl.DateTimeFormat("en-US", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      }).format(value),
    );
    expect(formatTime(value, "pt-BR")).toBe(
      new Intl.DateTimeFormat("pt-BR", { hour: "2-digit", minute: "2-digit" }).format(value),
    );
  });
});

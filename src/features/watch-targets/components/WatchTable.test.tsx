import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { WatchTarget } from "../../../shared/contracts";
import { I18nProvider } from "../../../shared/i18n";
import { WatchTable } from "./WatchTable";

const target: WatchTarget = {
  id: "target-1",
  name: "Example",
  url: "https://example.com/live",
  enabled: true,
  state: "Watching",
  statusDetail: "Waiting for live stream",
  nextCheckAt: null,
  lastCheckedAt: null,
  lastRecordingAt: null,
  activeJobId: null,
  createdAt: "2024-01-01T00:00:00Z",
  providerUserId: null,
  avatarUrl: null,
};

describe("WatchTable", () => {
  it("invokes row actions", async () => {
    const user = userEvent.setup();
    const onCheck = vi.fn();
    const onEdit = vi.fn();
    const onToggle = vi.fn();
    const onRemove = vi.fn();
    render(
      <I18nProvider locale="en">
        <WatchTable
          targets={[target]}
          selectedId={null}
          onSelect={vi.fn()}
          onCheck={onCheck}
          onEdit={onEdit}
          onToggle={onToggle}
          onRemove={onRemove}
        />
      </I18nProvider>,
    );

    await user.click(screen.getByRole("button", { name: "Check Example now" }));
    await user.click(screen.getByRole("button", { name: "Edit Example" }));
    await user.click(screen.getByRole("button", { name: "Pause Example" }));
    await user.click(screen.getByRole("button", { name: "Remove Example" }));

    expect(onCheck).toHaveBeenCalledWith(target);
    expect(onEdit).toHaveBeenCalledWith(target);
    expect(onToggle).toHaveBeenCalledWith(target);
    expect(onRemove).toHaveBeenCalledWith(target);
  });
});

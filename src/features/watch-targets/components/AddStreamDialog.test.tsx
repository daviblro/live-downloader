import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { WatchTarget } from "../../../shared/contracts";
import { I18nProvider } from "../../../shared/i18n";
import { AddStreamDialog } from "./AddStreamDialog";

describe("AddStreamDialog", () => {
  afterEach(cleanup);

  it("submits valid stream details and closes", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(
      <I18nProvider locale="en">
        <AddStreamDialog onClose={onClose} onSubmit={onSubmit} />
      </I18nProvider>,
    );

    await user.type(screen.getByLabelText("Source name"), "Example");
    await user.type(screen.getByLabelText("Stream URL"), "https://example.com/live");
    await user.click(screen.getByRole("button", { name: "Add stream" }));

    expect(onSubmit).toHaveBeenCalledWith({ name: "Example", url: "https://example.com/live" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("prefills and submits an existing stream", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const target: WatchTarget = {
      id: "target-1",
      name: "Old name",
      url: "https://www.twitch.tv/oldname",
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
    render(
      <I18nProvider locale="en">
        <AddStreamDialog target={target} onClose={vi.fn()} onSubmit={onSubmit} />
      </I18nProvider>,
    );

    expect(screen.getByRole("heading", { name: "Edit stream" })).toBeInTheDocument();
    const name = screen.getByLabelText("Source name");
    await user.clear(name);
    await user.type(name, "New name");
    await user.click(screen.getByRole("button", { name: "Save changes" }));

    expect(onSubmit).toHaveBeenCalledWith({
      name: "New name",
      url: "https://www.twitch.tv/oldname",
    });
  });
});

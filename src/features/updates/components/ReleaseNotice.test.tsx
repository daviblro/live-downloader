import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "../../../shared/i18n";
import { ReleaseNotice } from "./ReleaseNotice";

const release = {
  version: "1.3.0",
  url: "https://github.com/daviblro/live-downloader/releases/tag/v1.3.0",
};

describe("ReleaseNotice", () => {
  afterEach(cleanup);

  it("installs an updater-signed release from the app", async () => {
    const onInstall = vi.fn();
    render(
      <I18nProvider locale="en">
        <ReleaseNotice
          release={{ ...release, installable: true }}
          installState={{ phase: "idle" }}
          onInstall={onInstall}
          onOpen={vi.fn()}
          onDismiss={vi.fn()}
        />
      </I18nProvider>,
    );
    await userEvent.setup().click(screen.getByRole("button", { name: /update now/i }));
    expect(onInstall).toHaveBeenCalledOnce();
  });

  it("shows download progress and prevents dismissing while updating", () => {
    render(
      <I18nProvider locale="en">
        <ReleaseNotice
          release={{ ...release, installable: true }}
          installState={{ phase: "downloading", percent: 42 }}
          onInstall={vi.fn()}
          onOpen={vi.fn()}
          onDismiss={vi.fn()}
        />
      </I18nProvider>,
    );
    expect(screen.getByText("Downloading the update… 42%")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /updating/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /dismiss update notice/i })).toBeDisabled();
  });

  it("only links to GitHub for releases without updater metadata", () => {
    render(
      <I18nProvider locale="en">
        <ReleaseNotice
          release={{ ...release, installable: false }}
          installState={{ phase: "idle" }}
          onInstall={vi.fn()}
          onOpen={vi.fn()}
          onDismiss={vi.fn()}
        />
      </I18nProvider>,
    );
    expect(screen.queryByRole("button", { name: /update now/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /view release/i })).toBeInTheDocument();
  });
});

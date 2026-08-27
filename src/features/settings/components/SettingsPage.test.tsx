import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AppSettings } from "../../../shared/contracts";
import { I18nProvider } from "../../../shared/i18n";
import { SettingsPage } from "./SettingsPage";

const apiMock = vi.hoisted(() => ({
  twitchStatus: vi.fn(),
  startTwitchConnect: vi.fn(),
  openUrl: vi.fn(),
  pollTwitchConnect: vi.fn(),
  disconnectTwitch: vi.fn(),
}));

vi.mock("../../../shared/tauri/client", () => ({ api: apiMock, isDesktop: true }));

const settings: AppSettings = {
  locale: "en",
  theme: "system",
  downloadDirectory: "C:\\Downloads",
  probeIntervalSeconds: 60,
  maxConcurrentRecordings: 2,
  startWithWindows: false,
  notificationsEnabled: true,
  externalYtdlpPath: null,
};

describe("SettingsPage", () => {
  afterEach(cleanup);

  beforeEach(() => {
    vi.clearAllMocks();
    apiMock.twitchStatus.mockResolvedValue({ available: true, connected: false, login: null });
  });

  it("saves the edited settings", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockResolvedValue(undefined);
    render(
      <I18nProvider locale="en">
        <SettingsPage settings={settings} onSave={onSave} onLocalePreview={vi.fn()} />
      </I18nProvider>,
    );

    await user.clear(screen.getByLabelText("Concurrent recordings"));
    await user.type(screen.getByLabelText("Concurrent recordings"), "3");
    await user.click(screen.getByRole("button", { name: "Save changes" }));

    expect(onSave).toHaveBeenCalledWith({ ...settings, maxConcurrentRecordings: 3 });
  });

  it("keeps polling when Twitch cannot be opened automatically", async () => {
    const user = userEvent.setup();
    apiMock.startTwitchConnect.mockResolvedValue({
      deviceCode: "device-code",
      userCode: "ABCD1234",
      verificationUri: "https://www.twitch.tv/activate?device-code=ABCD1234",
      expiresIn: 600,
      interval: 1,
    });
    apiMock.openUrl.mockRejectedValue(new Error("Not allowed to open url"));
    apiMock.pollTwitchConnect.mockResolvedValue({ connected: true, login: "streamer" });

    render(
      <I18nProvider locale="en">
        <SettingsPage settings={settings} onSave={vi.fn()} onLocalePreview={vi.fn()} />
      </I18nProvider>,
    );

    const connect = await screen.findByRole("button", { name: "Connect Twitch" });
    await waitFor(() => expect(connect).toBeEnabled());
    await user.click(connect);

    await waitFor(() => expect(apiMock.pollTwitchConnect).toHaveBeenCalledWith("device-code"), {
      timeout: 2500,
    });
    expect(await screen.findByText("Connected as streamer")).toBeInTheDocument();
    expect(screen.queryByText("Not allowed to open url")).not.toBeInTheDocument();
  });
});

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { AppSettings } from "../../../shared/contracts";
import { I18nProvider } from "../../../shared/i18n";
import { SettingsPage } from "./SettingsPage";

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
});

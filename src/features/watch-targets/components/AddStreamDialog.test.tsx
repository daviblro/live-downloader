import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { I18nProvider } from "../../../shared/i18n";
import { AddStreamDialog } from "./AddStreamDialog";

describe("AddStreamDialog", () => {
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
});

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { PresenceCalendar } from "@/features/presence/presentation/components/presence-calendar";
import {
  PresenceContext,
  type PresenceContextValue,
} from "@/features/presence/presentation/context/presence-context";
import type { PresenceType } from "@/features/presence/domain/entities/presence";

describe("PresenceCalendar", () => {
  it("opens the day dialog and sets the chosen presence type for the clicked day", async () => {
    const calls: Array<{ day: number; type: PresenceType }> = [];
    const value: PresenceContextValue = {
      presencesByDay: new Map(),
      isLoading: false,
      isSubmitting: false,
      error: null,
      setPresence: (day, type) => {
        calls.push({ day, type });
        return Promise.resolve(true);
      },
      deletePresence: () => Promise.resolve(true),
      clearError: () => {},
    };

    const user = userEvent.setup();
    render(
      <PresenceContext.Provider value={value}>
        <PresenceCalendar />
      </PresenceContext.Provider>,
    );

    // The 15th is unique in the 6-week grid (adjacent months only show edge days).
    await user.click(screen.getByRole("button", { name: /15/ }));
    // The dialog offers the four types; pick Office (fr: "Bureau").
    await user.click(await screen.findByRole("button", { name: "Bureau" }));

    expect(calls).toHaveLength(1);
    expect(calls[0].type).toBe("office");
    // The day passed to setPresence is a canonical UTC-midnight key.
    expect(calls[0].day % 86_400_000).toBe(0);
  });
});

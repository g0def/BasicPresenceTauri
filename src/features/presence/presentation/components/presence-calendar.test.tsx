import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import {
  CommuteContext,
  type CommuteContextValue,
} from "@/features/commute/presentation/context/commute-context";
import type { PresenceType } from "@/features/presence/domain/entities/presence";
import { PresenceCalendar } from "@/features/presence/presentation/components/presence-calendar";
import {
  PresenceContext,
  type PresenceContextValue,
} from "@/features/presence/presentation/context/presence-context";
import { CellDisplayProvider } from "@/shared/cell-display/cell-display-provider";

const commuteValue: CommuteContextValue = {
  commutes: [],
  emissionFactors: [],
  isLoading: false,
  isSubmitting: false,
  error: null,
  createCommute: () => Promise.resolve(true),
  updateCommute: () => Promise.resolve(true),
  deleteCommute: () => Promise.resolve(true),
  clearError: () => {},
};

describe("PresenceCalendar", () => {
  it("opens the day dialog and sets a non-commute presence type for the clicked day", async () => {
    const calls: Array<{ day: number; type: PresenceType }> = [];
    const value: PresenceContextValue = {
      presencesByDay: new Map(),
      currentMonth: new Date(),
      setCurrentMonth: () => {},
      reload: () => Promise.resolve(),
      isLoading: false,
      isSubmitting: false,
      error: null,
      setPresence: (day, type) => {
        calls.push({ day, type });
        return Promise.resolve(true);
      },
      deletePresence: () => Promise.resolve(true),
      getPresenceTrips: () => Promise.resolve([]),
      importPresences: () => Promise.resolve(null),
      clearError: () => {},
    };

    const user = userEvent.setup();
    render(
      <CommuteContext.Provider value={commuteValue}>
        <PresenceContext.Provider value={value}>
          <CellDisplayProvider>
            <PresenceCalendar />
          </CellDisplayProvider>
        </PresenceContext.Provider>
      </CommuteContext.Provider>,
    );

    // The 15th is unique in the 6-week grid (adjacent months only show edge days).
    await user.click(screen.getByRole("button", { name: /15/ }));
    // Vacation ("Congés") is a non-commute type: it persists immediately.
    await user.click(await screen.findByRole("button", { name: "Congés" }));

    expect(calls).toHaveLength(1);
    expect(calls[0].type).toBe("vacation");
    // The day passed to setPresence is a canonical UTC-midnight key.
    expect(calls[0].day % 86_400_000).toBe(0);
  });
});

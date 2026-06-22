import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import {
  CommuteContext,
  type CommuteContextValue,
} from "@/features/commute/presentation/context/commute-context";
import type {
  Presence,
  PresenceType,
} from "@/features/presence/domain/entities/presence";
import { PresenceCalendar } from "@/features/presence/presentation/components/presence-calendar";
import {
  PresenceContext,
  type PresenceContextValue,
} from "@/features/presence/presentation/context/presence-context";
import { CellDisplayContext } from "@/shared/cell-display/cell-display-context";

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

function makePresence(day: number, type: PresenceType): Presence {
  return {
    id: `p-${day}`,
    profileId: "prof",
    day,
    type,
    co2Kg: null,
    isEstimated: false,
    createdAt: 0,
    updatedAt: 0,
    workMinutes: 0,
  };
}

function renderCalendar(value: PresenceContextValue) {
  return render(
    <CommuteContext.Provider value={commuteValue}>
      <PresenceContext.Provider value={value}>
        <CellDisplayContext.Provider value={{ mode: "co2", setMode: () => {} }}>
          <PresenceCalendar />
        </CellDisplayContext.Provider>
      </PresenceContext.Provider>
    </CommuteContext.Provider>,
  );
}

/** A context value whose `setPresence` records every call into `calls` (kept
 * to non-work types so the work-hours IPC module is never reached). */
function makeValue(
  presencesByDay: Map<number, Presence>,
  calls: Array<{ day: number; type: PresenceType }>,
): PresenceContextValue {
  return {
    presencesByDay,
    currentMonth: new Date(),
    setCurrentMonth: () => {},
    reload: () => Promise.resolve(),
    isLoading: false,
    isSubmitting: false,
    error: null,
    setPresence: (day, type) => {
      calls.push({ day, type });
      return Promise.resolve(makePresence(day, type));
    },
    deletePresence: () => Promise.resolve(true),
    getPresenceTrips: () => Promise.resolve([]),
    clearError: () => {},
  };
}

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
        return Promise.resolve(makePresence(day, type));
      },
      deletePresence: () => Promise.resolve(true),
      getPresenceTrips: () => Promise.resolve([]),
      clearError: () => {},
    };

    const user = userEvent.setup();
    renderCalendar(value);

    // The 15th is unique in the 6-week grid (adjacent months only show edge days).
    await user.click(screen.getByRole("button", { name: /15/ }));
    // Vacation ("Congés") is a non-commute type: it persists immediately.
    await user.click(await screen.findByRole("button", { name: "Congés" }));

    expect(calls).toHaveLength(1);
    expect(calls[0].type).toBe("vacation");
    // The day passed to setPresence is a canonical UTC-midnight key.
    expect(calls[0].day % 86_400_000).toBe(0);
  });

  it("copies a day, pastes onto empty days, and prompts before overwriting a filled day", async () => {
    const now = new Date();
    const day15 = Date.UTC(now.getFullYear(), now.getMonth(), 15);
    const day20 = Date.UTC(now.getFullYear(), now.getMonth(), 20);

    const calls: Array<{ day: number; type: PresenceType }> = [];
    const value = makeValue(
      new Map([[day15, makePresence(day15, "vacation")]]),
      calls,
    );

    const user = userEvent.setup();
    renderCalendar(value);

    // Arm the toggle, then pick the encoded 15th as the source day (anchored
    // regex avoids matching the year digits in other days' labels).
    await user.click(screen.getByRole("button", { name: "Copier un jour" }));
    await user.click(screen.getByRole("button", { name: /^15\b/ }));

    // The toggle flips to paste mode once a day is copied; copying alone
    // persists nothing.
    await screen.findByRole("button", { name: /remplir/i });
    expect(calls).toHaveLength(0);

    // Painting an empty day replays the copied presence onto it immediately.
    await user.click(screen.getByRole("button", { name: /^20\b/ }));
    expect(calls).toHaveLength(1);
    expect(calls[0]).toEqual({ day: day20, type: "vacation" });

    // Clicking a day that already has a presence asks for confirmation first
    // and persists nothing yet.
    await user.click(screen.getByRole("button", { name: /^15\b/ }));
    expect(await screen.findByText("Remplacer ce jour ?")).toBeTruthy();
    expect(calls).toHaveLength(1);
  });

  it("overwrites a filled day after confirmation, then auto-approves the rest of the session", async () => {
    const now = new Date();
    const day10 = Date.UTC(now.getFullYear(), now.getMonth(), 10);
    const day15 = Date.UTC(now.getFullYear(), now.getMonth(), 15);
    const day25 = Date.UTC(now.getFullYear(), now.getMonth(), 25);

    const calls: Array<{ day: number; type: PresenceType }> = [];
    const value = makeValue(
      new Map([
        [day10, makePresence(day10, "vacation")],
        [day15, makePresence(day15, "vacation")],
        [day25, makePresence(day25, "vacation")],
      ]),
      calls,
    );

    const user = userEvent.setup();
    renderCalendar(value);

    await user.click(screen.getByRole("button", { name: "Copier un jour" }));
    await user.click(screen.getByRole("button", { name: /^15\b/ }));
    await screen.findByRole("button", { name: /remplir/i });

    // First filled day → confirmation dialog; confirming overwrites it.
    await user.click(screen.getByRole("button", { name: /^10\b/ }));
    await user.click(await screen.findByRole("button", { name: "Remplacer" }));
    expect(calls).toEqual([{ day: day10, type: "vacation" }]);

    // A second filled day is now overwritten silently — no dialog.
    await user.click(screen.getByRole("button", { name: /^25\b/ }));
    expect(calls).toEqual([
      { day: day10, type: "vacation" },
      { day: day25, type: "vacation" },
    ]);
    expect(screen.queryByText("Remplacer ce jour ?")).toBeNull();
  });

  it("leaves the day unchanged and re-asks when the overwrite is cancelled", async () => {
    const now = new Date();
    const day10 = Date.UTC(now.getFullYear(), now.getMonth(), 10);
    const day15 = Date.UTC(now.getFullYear(), now.getMonth(), 15);

    const calls: Array<{ day: number; type: PresenceType }> = [];
    const value = makeValue(
      new Map([
        [day10, makePresence(day10, "vacation")],
        [day15, makePresence(day15, "vacation")],
      ]),
      calls,
    );

    const user = userEvent.setup();
    renderCalendar(value);

    await user.click(screen.getByRole("button", { name: "Copier un jour" }));
    await user.click(screen.getByRole("button", { name: /^15\b/ }));
    await screen.findByRole("button", { name: /remplir/i });

    // Cancelling the first overwrite persists nothing.
    await user.click(screen.getByRole("button", { name: /^10\b/ }));
    await user.click(await screen.findByRole("button", { name: "Annuler" }));
    expect(calls).toHaveLength(0);

    // Approval was not granted, so the next filled day prompts again.
    await user.click(screen.getByRole("button", { name: /^10\b/ }));
    expect(await screen.findByText("Remplacer ce jour ?")).toBeTruthy();
    expect(calls).toHaveLength(0);
  });
});

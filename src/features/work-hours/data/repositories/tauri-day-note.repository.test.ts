import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";

import { TauriDayNoteRepository } from "@/features/work-hours/data/repositories/tauri-day-note.repository";

describe("TauriDayNoteRepository", () => {
  it("fetches a day's note by presence id", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "get_day_note") {
        expect(args).toEqual({ presenceId: "p1" });
        return { markdown: "# Hi", html: "<h1>Hi</h1>" };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriDayNoteRepository();
    const note = await repo.get("p1");

    expect(note).toEqual({ markdown: "# Hi", html: "<h1>Hi</h1>" });
  });

  it("sends camelCase args and returns the backend-rendered HTML", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "set_day_note") {
        expect(args).toEqual({ presenceId: "p1", markdown: "# Hi" });
        return { markdown: "# Hi", html: "<h1>Hi</h1>" };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriDayNoteRepository();
    const saved = await repo.set("p1", "# Hi");

    expect(saved.html).toBe("<h1>Hi</h1>");
  });

  it("propagates a structured backend error (e.g. locked vault)", async () => {
    mockIPC(() => {
      throw { code: "SESSION_EXPIRED", message: "Session expirée" };
    });

    const repo = new TauriDayNoteRepository();
    await expect(repo.get("p1")).rejects.toMatchObject({
      code: "SESSION_EXPIRED",
    });
  });
});

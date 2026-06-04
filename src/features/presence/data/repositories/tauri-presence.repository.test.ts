import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";

import { TauriPresenceRepository } from "@/features/presence/data/repositories/tauri-presence.repository";

const DTO = {
  id: "x1",
  profileId: "p1",
  day: 1_717_200_000_000,
  type: "office" as const,
  createdAt: 1,
  updatedAt: 1,
};

describe("TauriPresenceRepository", () => {
  it("sends camelCase args (day + `type`) and maps the saved presence", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "set_presence") {
        expect(args).toEqual({
          profileId: "p1",
          day: 1_717_200_000_000,
          type: "office",
        });
        return DTO;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriPresenceRepository();
    const saved = await repo.set({
      profileId: "p1",
      day: 1_717_200_000_000,
      type: "office",
    });

    expect(saved.id).toBe("x1");
    expect(saved.type).toBe("office");
  });

  it("maps the presence list for a profile", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "list_presences") {
        expect(args).toEqual({ profileId: "p1" });
        return [DTO];
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriPresenceRepository();
    const list = await repo.listByProfile("p1");

    expect(list).toHaveLength(1);
    expect(list[0].day).toBe(1_717_200_000_000);
  });

  it("sends the id when removing and resolves void", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "delete_presence") {
        expect(args).toEqual({ id: "x1" });
        return null;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriPresenceRepository();
    await expect(repo.remove("x1")).resolves.toBeUndefined();
  });

  it("propagates a structured backend error (e.g. locked vault)", async () => {
    mockIPC(() => {
      throw { code: "SESSION_EXPIRED", message: "Session expirée" };
    });

    const repo = new TauriPresenceRepository();
    await expect(repo.listByProfile("p1")).rejects.toMatchObject({
      code: "SESSION_EXPIRED",
    });
  });
});

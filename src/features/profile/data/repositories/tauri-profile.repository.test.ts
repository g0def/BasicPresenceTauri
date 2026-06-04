import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";

import { TauriProfileRepository } from "@/features/profile/data/repositories/tauri-profile.repository";

describe("TauriProfileRepository", () => {
  it("sends camelCase args (poste defaulting to null) and maps the created profile", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "create_profile") {
        expect(args).toEqual({
          firstName: "Ada",
          lastName: "Lovelace",
          enterprise: "Analytical Engine",
          poste: null,
        });
        return {
          id: "p1",
          firstName: "Ada",
          lastName: "Lovelace",
          enterprise: "Analytical Engine",
          poste: null,
          createdAt: 1,
          updatedAt: 1,
        };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriProfileRepository();
    const profile = await repo.create({
      firstName: "Ada",
      lastName: "Lovelace",
      enterprise: "Analytical Engine",
    });

    expect(profile.id).toBe("p1");
    expect(profile.poste).toBeNull();
  });

  it("maps the profile list and active id", async () => {
    mockIPC((cmd) => {
      if (cmd === "list_profiles") {
        return {
          profiles: [
            {
              id: "p1",
              firstName: "Ada",
              lastName: "Lovelace",
              enterprise: "Analytical Engine",
              poste: null,
              createdAt: 1,
              updatedAt: 1,
            },
          ],
          activeProfileId: "p1",
        };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriProfileRepository();
    const list = await repo.list();

    expect(list.profiles).toHaveLength(1);
    expect(list.profiles[0].firstName).toBe("Ada");
    expect(list.activeProfileId).toBe("p1");
  });

  it("normalizes a structured backend error into an AppError", async () => {
    mockIPC(() => {
      throw { code: "NOT_FOUND", message: "Profil introuvable" };
    });

    const repo = new TauriProfileRepository();
    await expect(repo.setActive("missing")).rejects.toMatchObject({
      code: "NOT_FOUND",
    });
  });
});

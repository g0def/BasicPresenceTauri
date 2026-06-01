import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";

import { TauriAuthRepository } from "@/features/auth/data/repositories/tauri-auth.repository";

describe("TauriAuthRepository", () => {
  it("sends camelCase args and maps the login DTO to an AuthSession", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "login") {
        expect(args).toEqual({ username: "alice", password: "secret123" });
        return {
          token: "tok_123",
          expiresAt: 1000,
          user: { id: "u1", username: "alice", createdAt: 1, updatedAt: 2 },
        };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriAuthRepository();
    const session = await repo.login({
      username: "alice",
      password: "secret123",
    });

    expect(session).toEqual({
      token: "tok_123",
      expiresAt: 1000,
      user: { id: "u1", username: "alice", createdAt: 1, updatedAt: 2 },
    });
  });

  it("normalizes a structured backend error into an AppError", async () => {
    mockIPC(() => {
      // Tauri rejects with the serialized Err payload; ours is { code, message }.
      throw { code: "INVALID_CREDENTIALS", message: "Identifiants invalides" };
    });

    const repo = new TauriAuthRepository();
    await expect(
      repo.login({ username: "x", password: "y" }),
    ).rejects.toMatchObject({ code: "INVALID_CREDENTIALS" });
  });
});

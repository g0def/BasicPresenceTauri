import { describe, expect, it } from "vitest";

import {
  toAuthSession,
  toSessionStatus,
} from "@/features/auth/data/mappers/auth.mapper";

describe("auth.mapper", () => {
  it("maps a login response DTO to an AuthSession", () => {
    const session = toAuthSession({
      token: "t",
      expiresAt: 123,
      user: { id: "u1", username: "bob", createdAt: 1, updatedAt: 2 },
      failedAttemptsSinceLastLogin: 2,
      securityEvents: [{ kind: "account_locked", at: 100 }],
    });

    expect(session).toEqual({
      token: "t",
      expiresAt: 123,
      user: { id: "u1", username: "bob", createdAt: 1, updatedAt: 2 },
      securityNotice: {
        failedAttempts: 2,
        events: [{ kind: "account_locked", at: 100 }],
      },
    });
  });

  it("maps a session-status DTO", () => {
    expect(toSessionStatus({ valid: true, remainingMs: 42 })).toEqual({
      valid: true,
      remainingMs: 42,
    });
  });
});

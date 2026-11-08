import { describe, expect, it } from "vitest";

import { AppError, normalizeError } from "@/core/errors";

describe("normalizeError", () => {
  it("keeps the code and message of a backend `{ code, message }` error", () => {
    const e = normalizeError({ code: "SESSION_EXPIRED", message: "expired" });
    expect(e).toBeInstanceOf(AppError);
    expect(e.code).toBe("SESSION_EXPIRED");
    expect(e.message).toBe("expired");
  });

  it("maps an unknown code to UNKNOWN", () => {
    expect(normalizeError({ code: "WHATEVER", message: "m" }).code).toBe(
      "UNKNOWN",
    );
  });

  it("falls back to a translated message when the backend sends none", () => {
    const e = normalizeError({ code: "INTERNAL" });
    expect(e.code).toBe("INTERNAL");
    expect(e.message).toBe("Une erreur est survenue");
  });

  it("wraps strings and unknown values", () => {
    expect(normalizeError("boom")).toMatchObject({
      code: "UNKNOWN",
      message: "boom",
    });
    expect(normalizeError(42).code).toBe("UNKNOWN");
  });

  it("returns an AppError unchanged", () => {
    const original = new AppError("VALIDATION", "bad");
    expect(normalizeError(original)).toBe(original);
  });
});

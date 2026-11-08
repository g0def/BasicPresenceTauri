import { afterEach, describe, expect, it, vi } from "vitest";

import {
  createQueryClient,
  setSessionExpiredHandler,
} from "@/app/query-client";
import { AppError } from "@/core/errors";

describe("query client", () => {
  afterEach(() => setSessionExpiredHandler(null));

  it("logs out once, centrally, when a query hits a locked vault", async () => {
    const onExpired = vi.fn();
    setSessionExpiredHandler(onExpired);
    const client = createQueryClient();

    await expect(
      client.fetchQuery({
        queryKey: ["x"],
        queryFn: () =>
          Promise.reject(new AppError("SESSION_EXPIRED", "expired")),
      }),
    ).rejects.toThrow();

    expect(onExpired).toHaveBeenCalledOnce();
  });

  it("does the same for a mutation, and ignores other errors", async () => {
    const onExpired = vi.fn();
    setSessionExpiredHandler(onExpired);
    const client = createQueryClient();
    const run = (error: AppError) =>
      client
        .getMutationCache()
        .build(client, { mutationFn: () => Promise.reject(error) })
        .execute(undefined)
        .catch(() => undefined);

    await run(new AppError("VALIDATION", "bad input"));
    expect(onExpired).not.toHaveBeenCalled();

    await run(new AppError("SESSION_EXPIRED", "expired"));
    expect(onExpired).toHaveBeenCalledOnce();
  });

  it("never retries a failed backend call", async () => {
    const client = createQueryClient();
    const queryFn = vi.fn(() => Promise.reject(new AppError("INTERNAL", "x")));
    await expect(
      client.fetchQuery({ queryKey: ["y"], queryFn }),
    ).rejects.toThrow();
    expect(queryFn).toHaveBeenCalledOnce();
  });
});

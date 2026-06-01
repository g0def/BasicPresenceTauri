import { invoke as tauriInvoke } from "@tauri-apps/api/core";

import { normalizeError } from "@/core/errors";

/**
 * Typed wrapper around Tauri's `invoke` with error normalization.
 *
 * This is the ONLY module allowed to import `@tauri-apps/api`. Everything else
 * goes through a repository that calls this function.
 */
export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (raw) {
    throw normalizeError(raw);
  }
}

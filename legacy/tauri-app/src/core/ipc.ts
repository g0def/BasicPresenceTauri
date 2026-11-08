import { invoke as tauriInvoke } from "@tauri-apps/api/core";

import { normalizeError } from "@/core/errors";

/**
 * Registry of the backend commands: name → `{ args, result }`.
 *
 * Empty here on purpose: each feature declares its own commands next to its
 * DTOs (`features/<f>/data/commands.ts`) through module augmentation, so the
 * dependency keeps pointing from the features to `core`, never the reverse:
 *
 * ```ts
 * declare module "@/core/ipc" {
 *   interface CommandRegistry {
 *     list_profiles: { args: void; result: ProfilesDto };
 *   }
 * }
 * ```
 */
// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- augmented per feature
export interface CommandRegistry {}

/** Name of a registered backend command. */
export type CommandName = keyof CommandRegistry;

type ArgsOf<K extends CommandName> = CommandRegistry[K] extends {
  args: infer A;
}
  ? A
  : never;
type ResultOf<K extends CommandName> = CommandRegistry[K] extends {
  result: infer R;
}
  ? R
  : never;

/**
 * Typed wrapper around Tauri's `invoke` with error normalization: the command
 * name must be registered, its arguments and result are typed by the registry.
 *
 * This is the ONLY module allowed to import `@tauri-apps/api`. Everything else
 * goes through a repository that calls this function.
 */
export async function invoke<K extends CommandName>(
  command: K,
  ...args: ArgsOf<K> extends void ? [] : [ArgsOf<K>]
): Promise<ResultOf<K>> {
  try {
    return await tauriInvoke<ResultOf<K>>(
      command,
      args[0] as Record<string, unknown> | undefined,
    );
  } catch (raw) {
    throw normalizeError(raw);
  }
}

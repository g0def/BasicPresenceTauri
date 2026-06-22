import { openUrl } from "@tauri-apps/plugin-opener";

/**
 * Open a URL in the user's default browser (Tauri opener plugin).
 *
 * This is the ONLY module allowed to import `@tauri-apps/plugin-opener`
 * (architecture guard, mirroring `src/core/ipc.ts`). External navigation never
 * happens inside the webview.
 */
export async function openExternal(url: string): Promise<void> {
  // Best-effort: opening a link must never surface an error to the UI. The catch
  // also avoids an unhandled promise rejection at the fire-and-forget call site.
  try {
    await openUrl(url);
  } catch {
    // Intentionally ignored (e.g. no default browser / missing xdg-open).
  }
}

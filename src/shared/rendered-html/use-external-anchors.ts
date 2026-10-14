import { useCallback } from "react";

import { openExternal } from "@/core/external-link";

/**
 * Route clicks on links inside backend-rendered HTML to the OS browser.
 *
 * Rendered Markdown really does contain `<a href>` anchors: comrak's autolink
 * turns a pasted URL into one, and the Ammonia whitelist keeps http/https/mailto.
 * Left alone, clicking one navigates the WebView itself — a window with no
 * address bar showing a remote page, which is exactly the shape of a phishing
 * prompt. Notes can arrive from an imported bundle, so the content is not
 * necessarily the user's own.
 *
 * A `on_navigation` guard in the Rust builder is the real enforcement (it holds
 * whatever the frontend does). This handler is what makes the link still *work*,
 * by sending it to the browser instead of silently doing nothing.
 *
 * Returns a click handler to spread on the container that owns the rendered
 * HTML. Uses event delegation, so it covers anchors that appear later.
 */
export function useExternalAnchors() {
  return useCallback((event: React.MouseEvent<HTMLElement>) => {
    const anchor = (event.target as HTMLElement | null)?.closest?.("a[href]");
    if (!anchor) return;

    const href = anchor.getAttribute("href");
    if (!href) return;

    // In-document jumps (heading anchors) stay in the WebView.
    if (href.startsWith("#")) return;

    // Anything else leaves the app: never in this window.
    event.preventDefault();
    // Stop the container's own click handler (e.g. "edit this note") from
    // firing as well — clicking a link is not a request to edit.
    event.stopPropagation();

    // Only hand the OS schemes we expect. Ammonia already strips the rest, but
    // this module must not depend on that to stay safe.
    if (/^(https?|mailto):/i.test(href)) {
      void openExternal(href);
    }
  }, []);
}

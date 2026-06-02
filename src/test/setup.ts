import "@testing-library/jest-dom/vitest";
import { afterEach, beforeEach } from "vitest";
import { cleanup } from "@testing-library/react";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";

import i18n from "@/core/i18n";

// Radix UI (used by the shadcn Select) relies on a few DOM APIs that jsdom
// does not implement. Polyfill them so the components can be driven in tests.
if (!Element.prototype.hasPointerCapture) {
  Element.prototype.hasPointerCapture = () => false;
}
if (!Element.prototype.setPointerCapture) {
  Element.prototype.setPointerCapture = () => {};
}
if (!Element.prototype.releasePointerCapture) {
  Element.prototype.releasePointerCapture = () => {};
}
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {};
}
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}

// Reset the DOM and Tauri mocks between tests so state never leaks.
afterEach(() => {
  cleanup();
  clearMocks();
});

// Safe default: every command rejects unless a test overrides it via mockIPC,
// so no test accidentally hits a real (absent) backend. Pin the language to FR
// so component assertions on French strings stay deterministic regardless of
// the jsdom navigator locale.
beforeEach(async () => {
  mockIPC((cmd) => {
    return Promise.reject(
      new Error(`Unmocked Tauri command in test: "${cmd}"`),
    );
  });
  await i18n.changeLanguage("fr");
});

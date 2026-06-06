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

// jsdom's default (opaque) origin does not expose a usable `localStorage`, so
// any code reading/writing it throws. Provide an in-memory polyfill.
let hasLocalStorage = false;
try {
  hasLocalStorage =
    typeof globalThis.localStorage !== "undefined" &&
    globalThis.localStorage !== null;
} catch {
  hasLocalStorage = false;
}
if (!hasLocalStorage) {
  class MemoryStorage implements Storage {
    private store = new Map<string, string>();
    get length(): number {
      return this.store.size;
    }
    clear(): void {
      this.store.clear();
    }
    getItem(key: string): string | null {
      return this.store.has(key) ? this.store.get(key)! : null;
    }
    setItem(key: string, value: string): void {
      this.store.set(key, String(value));
    }
    removeItem(key: string): void {
      this.store.delete(key);
    }
    key(index: number): string | null {
      return Array.from(this.store.keys())[index] ?? null;
    }
  }
  Object.defineProperty(globalThis, "localStorage", {
    value: new MemoryStorage(),
    configurable: true,
    writable: true,
  });
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

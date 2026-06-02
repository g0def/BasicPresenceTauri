import "@testing-library/jest-dom/vitest";
import { afterEach, beforeEach } from "vitest";
import { cleanup } from "@testing-library/react";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";

import i18n from "@/core/i18n";

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

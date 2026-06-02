import { useCallback, useEffect, useState } from "react";

export type Theme = "light" | "dark";

const STORAGE_KEY = "theme";

function readStoredTheme(): Theme {
  if (typeof localStorage === "undefined") return "light";
  return localStorage.getItem(STORAGE_KEY) === "dark" ? "dark" : "light";
}

/**
 * Applies the persisted theme to <html> synchronously at startup, before React
 * renders, so dark-mode users never see a flash of the light theme. Call once
 * from the entry point.
 */
export function applyStoredTheme(): void {
  document.documentElement.classList.toggle("dark", readStoredTheme() === "dark");
}

/**
 * Manual light/dark theme toggle. Light by default, choice persisted in
 * localStorage. Applies the `.dark` class on <html> so the palette's
 * `@custom-variant dark` kicks in.
 */
export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(readStoredTheme);

  // Keep the DOM in sync with the current theme. Persistence happens on change
  // (in setTheme/toggleTheme) so we never write on mount.
  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  const setTheme = useCallback((next: Theme) => {
    localStorage.setItem(STORAGE_KEY, next);
    setThemeState(next);
  }, []);

  const toggleTheme = useCallback(
    () => setThemeState((prev) => {
      const next: Theme = prev === "dark" ? "light" : "dark";
      localStorage.setItem(STORAGE_KEY, next);
      return next;
    }),
    [],
  );

  return { theme, setTheme, toggleTheme };
}

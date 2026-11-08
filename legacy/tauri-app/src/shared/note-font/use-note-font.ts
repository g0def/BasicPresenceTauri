import { useCallback, useEffect, useState } from "react";

import { NOTE_FONTS, type NoteFont } from "./note-font";

export { NOTE_FONTS };
export type { NoteFont };

// Note font is now owned per profile (see profile-settings). This localStorage
// key is the device-level "last applied" cache: it lets the pre-React paint pick
// a sensible font before the active profile's settings have loaded.
const STORAGE_KEY = "note-font";

function readStoredNoteFont(): NoteFont {
  if (typeof localStorage === "undefined") return "sans";
  const stored = localStorage.getItem(STORAGE_KEY);
  return NOTE_FONTS.includes(stored as NoteFont)
    ? (stored as NoteFont)
    : "sans";
}

/**
 * Applies the cached note font to <html> (as `data-note-font`) before React
 * renders, so the notes editor/preview use a font from the first paint. The
 * authoritative per-profile value is reconciled afterwards by
 * `ProfileSettingsProvider`. Call once from the entry point.
 */
export function applyStoredNoteFont(): void {
  document.documentElement.dataset.noteFont = readStoredNoteFont();
}

/**
 * Apply a note font to <html> and refresh the pre-React paint cache. Used by
 * `ProfileSettingsProvider` to push the active profile's font to the DOM.
 */
export function applyNoteFont(font: NoteFont): void {
  document.documentElement.dataset.noteFont = font;
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STORAGE_KEY, font);
  }
}

/**
 * Display font for day notes (the Markdown editor + the rendered view). The
 * choice is persisted in localStorage and applied via the `data-note-font`
 * attribute on <html>, which drives the `--note-font` CSS variable used by the
 * note components — so changing it restyles the notes without a re-render.
 */
export function useNoteFont() {
  const [noteFont, setNoteFontState] = useState<NoteFont>(readStoredNoteFont);

  useEffect(() => {
    document.documentElement.dataset.noteFont = noteFont;
  }, [noteFont]);

  const setNoteFont = useCallback((next: NoteFont) => {
    localStorage.setItem(STORAGE_KEY, next);
    setNoteFontState(next);
  }, []);

  return { noteFont, setNoteFont };
}

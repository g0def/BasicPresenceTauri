import { useCallback, useEffect, useState } from "react";

export type NoteFont = "sans" | "serif" | "mono";

const STORAGE_KEY = "note-font";
export const NOTE_FONTS: NoteFont[] = ["sans", "serif", "mono"];

function readStoredNoteFont(): NoteFont {
  if (typeof localStorage === "undefined") return "sans";
  const stored = localStorage.getItem(STORAGE_KEY);
  return NOTE_FONTS.includes(stored as NoteFont)
    ? (stored as NoteFont)
    : "sans";
}

/**
 * Applies the persisted note font to <html> (as `data-note-font`) before React
 * renders, so the notes editor/preview use the chosen font from the first paint.
 * Call once from the entry point.
 */
export function applyStoredNoteFont(): void {
  document.documentElement.dataset.noteFont = readStoredNoteFont();
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

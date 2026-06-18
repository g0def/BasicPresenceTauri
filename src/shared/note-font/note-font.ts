/** Note-font choices, as a framework-agnostic type (importable from a domain
 * layer without pulling in React). The React hook/applier live alongside in
 * `use-note-font.ts` and re-export these. */
export type NoteFont = "sans" | "serif" | "mono";

export const NOTE_FONTS: NoteFont[] = ["sans", "serif", "mono"];

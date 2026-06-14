import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import {
  applyStoredNoteFont,
  useNoteFont,
} from "@/shared/note-font/use-note-font";

describe("useNoteFont", () => {
  beforeEach(() => {
    localStorage.clear();
    delete document.documentElement.dataset.noteFont;
  });

  afterEach(() => {
    delete document.documentElement.dataset.noteFont;
  });

  it("defaults to sans and applies it to <html>", () => {
    const { result } = renderHook(() => useNoteFont());
    expect(result.current.noteFont).toBe("sans");
    expect(document.documentElement.dataset.noteFont).toBe("sans");
  });

  it("sets a font, persists it, and reflects it on <html>", () => {
    const { result } = renderHook(() => useNoteFont());

    act(() => result.current.setNoteFont("mono"));

    expect(result.current.noteFont).toBe("mono");
    expect(localStorage.getItem("note-font")).toBe("mono");
    expect(document.documentElement.dataset.noteFont).toBe("mono");
  });

  it("applyStoredNoteFont restores a persisted font on startup", () => {
    localStorage.setItem("note-font", "serif");
    applyStoredNoteFont();
    expect(document.documentElement.dataset.noteFont).toBe("serif");
  });

  it("falls back to sans for an unknown stored value", () => {
    localStorage.setItem("note-font", "comic-sans");
    const { result } = renderHook(() => useNoteFont());
    expect(result.current.noteFont).toBe("sans");
  });
});

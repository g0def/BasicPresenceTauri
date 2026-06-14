import { useCallback, useEffect, useRef, useState } from "react";

import { isAppError } from "@/core/errors";
import { TauriDayNoteRepository } from "@/features/work-hours/data/repositories/tauri-day-note.repository";
import { makeGetDayNoteUseCase } from "@/features/work-hours/domain/use-cases/get-day-note";
import { makeSetDayNoteUseCase } from "@/features/work-hours/domain/use-cases/set-day-note";

const repo = new TauriDayNoteRepository();
const getDayNote = makeGetDayNoteUseCase(repo);
const setDayNote = makeSetDayNoteUseCase(repo);

/** Delay between the last keystroke and the persisting call. */
const SAVE_DEBOUNCE_MS = 600;

export interface UseDayNoteValue {
  /** Raw Markdown source (the editor is seeded with this). */
  markdown: string;
  /** Backend-rendered, sanitized HTML (shown in read-only view). */
  html: string;
  isLoading: boolean;
  /** `true` while a save is pending or in flight (subtle indicator). */
  isSaving: boolean;
  error: string | null;
  /** Update the Markdown (from the editor); persisted debounced. */
  setMarkdown: (markdown: string) => void;
  /** Persist any pending edit immediately (used when leaving edit mode). */
  flush: () => Promise<void>;
}

/**
 * Local state of a day's Markdown note with debounced live persistence: every
 * keystroke applies instantly, then the note is saved once edits settle. The
 * backend returns the rendered, sanitized HTML which we adopt for the read-only
 * view. Pending edits are flushed on unmount so navigating away never loses one.
 */
export function useDayNote(
  presenceId: string,
  onSessionExpired?: () => void,
): UseDayNoteValue {
  const [markdown, setMarkdownState] = useState("");
  const [html, setHtml] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const markdownRef = useRef("");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dirtyRef = useRef(false);
  // Bumped on every local edit; a save response is only applied when no newer
  // edit happened while the call was in flight.
  const versionRef = useRef(0);

  const onSessionExpiredRef = useRef(onSessionExpired);
  onSessionExpiredRef.current = onSessionExpired;

  const handleError = useCallback((e: unknown) => {
    if (isAppError(e) && e.code === "SESSION_EXPIRED") {
      onSessionExpiredRef.current?.();
      return;
    }
    setError(isAppError(e) ? e.message : "Une erreur est survenue");
  }, []);

  const persist = useCallback(async () => {
    if (!dirtyRef.current) return;
    dirtyRef.current = false;
    const version = versionRef.current;
    try {
      const saved = await setDayNote(presenceId, markdownRef.current);
      // Adopt the rendered HTML only if no newer edit happened meanwhile.
      if (versionRef.current === version) {
        setHtml(saved.html);
        setError(null);
      }
    } catch (e) {
      dirtyRef.current = true;
      handleError(e);
    } finally {
      if (versionRef.current === version) setIsSaving(false);
    }
  }, [presenceId, handleError]);

  const scheduleSave = useCallback(() => {
    dirtyRef.current = true;
    setIsSaving(true);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      timerRef.current = null;
      void persist();
    }, SAVE_DEBOUNCE_MS);
  }, [persist]);

  const setMarkdown = useCallback(
    (next: string) => {
      // The editor re-emits unchanged Markdown (e.g. right after mount); ignore
      // it so we don't schedule a redundant save.
      if (next === markdownRef.current) return;
      versionRef.current += 1;
      markdownRef.current = next;
      setMarkdownState(next);
      scheduleSave();
    },
    [scheduleSave],
  );

  const flush = useCallback(async () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    await persist();
  }, [persist]);

  // Load the day's note; flush any pending save on leave.
  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    getDayNote(presenceId)
      .then((note) => {
        if (cancelled) return;
        markdownRef.current = note.markdown;
        setMarkdownState(note.markdown);
        setHtml(note.html);
      })
      .catch((e) => {
        if (!cancelled) handleError(e);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      void persist();
    };
  }, [presenceId, persist, handleError]);

  return { markdown, html, isLoading, isSaving, error, setMarkdown, flush };
}

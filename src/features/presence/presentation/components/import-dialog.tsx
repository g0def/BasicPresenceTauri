import { useEffect, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  findConflicts,
  parseOldExport,
  type InvalidReason,
  type InvalidRow,
} from "@/features/presence/data/import/parse-old-export";
import type {
  ImportPresenceEntry,
  ImportSummary,
} from "@/features/presence/domain/entities/presence";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";

interface ImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

type Stage = "idle" | "review" | "result";

/** Max invalid rows listed before collapsing into a "…and N more" line. */
const MAX_INVALID_SHOWN = 10;

/**
 * Import presences from a JSON file exported by the old app. Flow: pick a file
 * → parse + map (separating valid rows from invalid ones) → review (listing any
 * invalid rows, and prompting Skip/Replace when days already exist) → import →
 * show a result summary. Targets the active profile (read via `usePresence`).
 */
export function ImportDialog({ open, onOpenChange }: ImportDialogProps) {
  const { t } = useTranslation();
  const { presencesByDay, importPresences, isSubmitting, error, clearError } =
    usePresence();

  const inputRef = useRef<HTMLInputElement>(null);
  const [stage, setStage] = useState<Stage>("idle");
  const [localError, setLocalError] = useState<string | null>(null);
  const [entries, setEntries] = useState<ImportPresenceEntry[]>([]);
  const [invalid, setInvalid] = useState<InvalidRow[]>([]);
  const [conflictCount, setConflictCount] = useState(0);
  const [summary, setSummary] = useState<ImportSummary | null>(null);

  // Reset everything whenever the dialog (re)opens.
  useEffect(() => {
    if (!open) return;
    setStage("idle");
    setLocalError(null);
    setEntries([]);
    setInvalid([]);
    setConflictCount(0);
    setSummary(null);
    clearError();
    if (inputRef.current) inputRef.current.value = "";
  }, [open, clearError]);

  const onFileChange = async (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    // Clear the value so re-selecting the same file fires onChange again.
    e.target.value = "";
    if (!file) return;

    let parsed: unknown;
    try {
      const raw = await file.text();
      // Strip a leading UTF-8 BOM if present (common in exported files).
      const text = raw.charCodeAt(0) === 0xfeff ? raw.slice(1) : raw;
      parsed = JSON.parse(text);
    } catch {
      setLocalError(t("import.errors.malformed"));
      return;
    }

    const result = parseOldExport(parsed);
    if (result.fatal === "notArray") {
      setLocalError(t("import.errors.notArray"));
      return;
    }
    if (result.entries.length === 0 && result.invalid.length === 0) {
      setLocalError(t("import.errors.empty"));
      return;
    }

    const existingDays = new Set(presencesByDay.keys());
    setEntries(result.entries);
    setInvalid(result.invalid);
    setConflictCount(findConflicts(result.entries, existingDays).length);
    setLocalError(null);
    setStage("review");
  };

  const runImport = async (replaceExisting: boolean) => {
    const result = await importPresences(entries, replaceExisting);
    if (result) {
      setSummary(result);
      setStage("result");
    }
    // On failure, the provider's `error` is shown and we stay on the review step.
  };

  const reasonText = (reason: InvalidReason): string => {
    switch (reason.code) {
      case "missingDate":
        return t("import.invalid.missingDate");
      case "badDate":
        return t("import.invalid.badDate", { value: reason.value });
      case "missingMode":
        return t("import.invalid.missingMode");
      case "unknownMode":
        return t("import.invalid.unknownMode", { value: reason.value });
    }
  };

  const feedback = localError ?? error;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("import.title")}</DialogTitle>
          {stage === "idle" && (
            <DialogDescription>{t("import.description")}</DialogDescription>
          )}
        </DialogHeader>

        {/* Hidden native file picker (opens the OS dialog in the Tauri webview). */}
        <input
          ref={inputRef}
          type="file"
          accept=".json,application/json"
          className="hidden"
          onChange={(e) => void onFileChange(e)}
        />

        {stage === "review" && (
          <div className="grid gap-3 text-sm">
            <p>{t("import.review.valid", { count: entries.length })}</p>
            {conflictCount > 0 && (
              <p>{t("import.review.conflicts", { count: conflictCount })}</p>
            )}
            {entries.length === 0 && (
              <p className="text-muted-foreground">
                {t("import.review.nothingValid")}
              </p>
            )}
            {invalid.length > 0 && (
              <div className="grid gap-1">
                <p className="font-medium">{t("import.invalid.heading")}</p>
                <ul className="max-h-40 list-disc overflow-auto pl-5 text-muted-foreground">
                  {invalid.slice(0, MAX_INVALID_SHOWN).map((r) => (
                    <li key={r.row}>
                      {(r.date ?? t("import.invalid.row", { row: r.row })) +
                        " — " +
                        reasonText(r.reason)}
                    </li>
                  ))}
                </ul>
                {invalid.length > MAX_INVALID_SHOWN && (
                  <p>
                    {t("import.invalid.more", {
                      count: invalid.length - MAX_INVALID_SHOWN,
                    })}
                  </p>
                )}
              </div>
            )}
          </div>
        )}

        {stage === "result" && summary && (
          <div className="grid gap-2 text-sm">
            <p>
              {t("import.result", {
                imported: summary.imported,
                replaced: summary.replaced,
                skipped: summary.skipped,
                total: summary.total,
              })}
            </p>
            {invalid.length > 0 && (
              <p className="text-muted-foreground">
                {t("import.resultInvalid", { count: invalid.length })}
              </p>
            )}
          </div>
        )}

        {feedback && <p className="text-sm text-destructive">{feedback}</p>}

        <DialogFooter>
          {stage === "idle" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("import.cancel")}
              </Button>
              <Button onClick={() => inputRef.current?.click()}>
                {t("import.chooseFile")}
              </Button>
            </>
          )}

          {stage === "review" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("import.cancel")}
              </Button>
              {entries.length > 0 &&
                (conflictCount > 0 ? (
                  <>
                    <Button
                      variant="outline"
                      disabled={isSubmitting}
                      onClick={() => void runImport(false)}
                    >
                      {t("import.skip")}
                    </Button>
                    <Button
                      disabled={isSubmitting}
                      onClick={() => void runImport(true)}
                    >
                      {t("import.replace")}
                    </Button>
                  </>
                ) : (
                  <Button
                    disabled={isSubmitting}
                    onClick={() => void runImport(false)}
                  >
                    {isSubmitting
                      ? t("import.importing")
                      : t("import.importValid", { count: entries.length })}
                  </Button>
                ))}
            </>
          )}

          {stage === "result" && (
            <Button onClick={() => onOpenChange(false)}>
              {t("import.done")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

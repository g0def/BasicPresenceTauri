import { lazy, Suspense, useState } from "react";
import { Check, Pencil } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { useDayNote } from "@/features/work-hours/presentation/hooks/use-day-note";
import { cn } from "@/lib/utils";

interface DayNoteProps {
  presenceId: string;
  onSessionExpired?: () => void;
}

// Milkdown (Crepe, ProseMirror-based) is heavy; load it only when the user
// actually edits — the read-only view renders the backend HTML without it.
const DayNoteEditor = lazy(() =>
  import("@/features/work-hours/presentation/components/day-note-editor").then(
    (m) => ({ default: m.DayNoteEditor }),
  ),
);

/**
 * A day's free-form note, shown full-width below the work-hours grid. The
 * read-only view renders the backend-sanitized HTML (with server-highlighted
 * code); the **Modifier / Terminé** button toggles the Milkdown WYSIWYG editor.
 *
 * Editing is an explicit mode (not blur-driven): Crepe's slash menu, toolbar and
 * tooltips render in portals outside the editable area, so a blur-to-exit would
 * close the editor the moment you click `/code` or a toolbar button.
 */
export function DayNote({ presenceId, onSessionExpired }: DayNoteProps) {
  const { t } = useTranslation();
  const { markdown, html, isLoading, isSaving, error, setMarkdown, flush } =
    useDayNote(presenceId, onSessionExpired);
  const [editing, setEditing] = useState(false);

  const startEditing = () => setEditing(true);
  const stopEditing = () => void flush().finally(() => setEditing(false));

  return (
    <section className="flex flex-col gap-2 rounded-xl border bg-card/40 p-4">
      <div className="flex items-center gap-2">
        <h3 className="text-sm font-semibold">{t("workHours.note.title")}</h3>
        <span
          className={cn(
            "ml-auto text-xs text-muted-foreground transition-opacity duration-300",
            isSaving ? "opacity-100" : "opacity-0",
          )}
          aria-live="polite"
        >
          {t("workHours.saving")}
        </span>
        {!isLoading && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={editing ? stopEditing : startEditing}
          >
            {editing ? (
              <>
                <Check className="size-4" />
                {t("workHours.note.done")}
              </>
            ) : (
              <>
                <Pencil className="size-4" />
                {t("workHours.note.edit")}
              </>
            )}
          </Button>
        )}
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {isLoading ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : editing ? (
        <Suspense
          fallback={
            <p className="text-sm text-muted-foreground">
              {t("common.loading")}
            </p>
          }
        >
          <DayNoteEditor
            defaultValue={markdown}
            placeholder={t("workHours.note.placeholder")}
            onChange={setMarkdown}
          />
        </Suspense>
      ) : html.trim() ? (
        <div
          role="button"
          tabIndex={0}
          onClick={startEditing}
          onKeyDown={(e) => {
            if (e.key === "Enter") startEditing();
          }}
          className="note-body prose dark:prose-invert max-w-none cursor-text rounded-md px-1 py-0.5 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          // Rendered + sanitized backend-side (comrak unsafe_=false + ammonia).
          dangerouslySetInnerHTML={{ __html: html }}
        />
      ) : (
        <button
          type="button"
          onClick={startEditing}
          className="self-start text-sm text-muted-foreground italic hover:text-foreground"
        >
          {t("workHours.note.empty")}
        </button>
      )}
    </section>
  );
}

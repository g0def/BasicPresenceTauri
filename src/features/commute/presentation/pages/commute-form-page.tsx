import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { ChevronLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type {
  Commute,
  CommuteSegment,
} from "@/features/commute/domain/entities/commute";
import { SegmentEditor } from "@/features/commute/presentation/components/segment-editor";
import { useCommute } from "@/features/commute/presentation/hooks/use-commute";

interface CommuteFormPageProps {
  /** When provided, the page edits this commute; otherwise it creates one. */
  commute?: Commute;
  /** Navigate back to the list (called on save success and cancel). */
  onDone: () => void;
}

const emptySegment = (): CommuteSegment => ({
  modeId: "",
  distanceKm: 0,
  occupants: 1,
});

/** Create/edit page for a commute preset. Router-free on purpose: the route
 * owns navigation through `onDone`, and a fresh mount per navigation makes
 * plain useState initializers enough (no reset-on-open effect needed). */
export function CommuteFormPage({ commute, onDone }: CommuteFormPageProps) {
  const { t } = useTranslation();
  const {
    emissionFactors,
    createCommute,
    updateCommute,
    isSubmitting,
    error,
    clearError,
  } = useCommute();
  const isEdit = Boolean(commute);

  const [name, setName] = useState(() => commute?.name ?? "");
  const [roundTrip, setRoundTrip] = useState(() => commute?.roundTrip ?? true);
  const [segments, setSegments] = useState<CommuteSegment[]>(() =>
    commute && commute.segments.length > 0
      ? commute.segments.map((s) => ({ ...s }))
      : [emptySegment()],
  );
  const [validationError, setValidationError] = useState<string | null>(null);

  // Drop any backend error left over from a previous screen.
  useEffect(() => {
    clearError();
  }, [clearError]);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setValidationError(t("commute.validation.nameRequired"));
      return;
    }
    const cleaned = segments.filter((s) => s.modeId);
    if (cleaned.length === 0) {
      setValidationError(t("commute.validation.minSegment"));
      return;
    }
    if (cleaned.some((s) => !(s.distanceKm > 0))) {
      setValidationError(t("commute.validation.distancePositive"));
      return;
    }

    const input = { name: name.trim(), roundTrip, segments: cleaned };
    const ok =
      isEdit && commute
        ? await updateCommute({ id: commute.id, ...input })
        : await createCommute(input);
    if (ok) onDone();
  };

  return (
    <section className="mt-6 flex w-full max-w-2xl flex-col gap-4 self-center">
      <div className="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          onClick={onDone}
          aria-label={t("commute.step.back")}
        >
          <ChevronLeft />
        </Button>
        <h2 className="text-xl font-semibold">
          {isEdit ? t("commute.form.editTitle") : t("commute.form.addTitle")}
        </h2>
      </div>
      <p className="text-sm text-muted-foreground">
        {t("commute.form.subtitle")}
      </p>

      <form onSubmit={onSubmit} className="grid gap-4">
        <div className="grid gap-2">
          <Label htmlFor="commute-name">{t("commute.form.name")}</Label>
          <Input
            id="commute-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            autoFocus
          />
        </div>

        <SegmentEditor
          segments={segments}
          onChange={setSegments}
          emissionFactors={emissionFactors}
          disabled={isSubmitting}
        />

        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={roundTrip}
            onChange={(e) => setRoundTrip(e.target.checked)}
            className="size-4 rounded border-input accent-primary"
          />
          {t("commute.form.roundTrip")}
        </label>

        {(validationError ?? error) && (
          <p className="text-sm text-destructive">{validationError ?? error}</p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onDone}>
            {t("commute.form.cancel")}
          </Button>
          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? t("commute.form.saving") : t("commute.form.save")}
          </Button>
        </div>
      </form>
    </section>
  );
}

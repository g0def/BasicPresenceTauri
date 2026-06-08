import { useEffect, useState } from "react";
import type { FormEvent } from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type {
  Commute,
  CommuteSegment,
} from "@/features/commute/domain/entities/commute";
import { SegmentEditor } from "@/features/commute/presentation/components/segment-editor";
import { useCommute } from "@/features/commute/presentation/hooks/use-commute";

interface CommuteFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** When provided, the dialog edits this commute; otherwise it creates one. */
  commute?: Commute | null;
}

const emptySegment = (): CommuteSegment => ({
  modeId: "",
  distanceKm: 0,
  occupants: 1,
});

export function CommuteFormDialog({
  open,
  onOpenChange,
  commute,
}: CommuteFormDialogProps) {
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

  const [name, setName] = useState("");
  const [roundTrip, setRoundTrip] = useState(true);
  const [segments, setSegments] = useState<CommuteSegment[]>([emptySegment()]);
  const [validationError, setValidationError] = useState<string | null>(null);

  // Reset whenever the dialog (re)opens or the edited commute changes.
  useEffect(() => {
    if (!open) return;
    setName(commute?.name ?? "");
    setRoundTrip(commute?.roundTrip ?? true);
    setSegments(
      commute && commute.segments.length > 0
        ? commute.segments.map((s) => ({ ...s }))
        : [emptySegment()],
    );
    setValidationError(null);
    clearError();
  }, [open, commute, clearError]);

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
    if (ok) onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form onSubmit={onSubmit} className="grid gap-4">
          <DialogHeader>
            <DialogTitle>
              {isEdit
                ? t("commute.form.editTitle")
                : t("commute.form.addTitle")}
            </DialogTitle>
            <DialogDescription>{t("commute.form.subtitle")}</DialogDescription>
          </DialogHeader>

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
            <p className="text-sm text-destructive">
              {validationError ?? error}
            </p>
          )}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              {t("commute.form.cancel")}
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? t("commute.form.saving") : t("commute.form.save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

import { useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { format } from "date-fns";
import { enUS, fr } from "date-fns/locale";
import { Clock4 } from "lucide-react";
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
  commuteToTrips,
  type CommuteSegment,
} from "@/features/commute/domain/entities/commute";
import { SegmentEditor } from "@/features/commute/presentation/components/segment-editor";
import {
  formatCo2,
  summarizeSegments,
} from "@/features/commute/presentation/commute-format";
import { useCommute } from "@/features/commute/presentation/hooks/use-commute";
import {
  PRESENCE_TYPES,
  type PresenceTrip,
  type PresenceType,
} from "@/features/presence/domain/entities/presence";
import { dayKey } from "@/features/presence/presentation/day-key";
import { PRESENCE_STYLES } from "@/features/presence/presentation/components/presence-colors";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import { cn } from "@/lib/utils";

interface PresenceDayDialogProps {
  /** The calendar day being edited, or `null` when the dialog is closed. */
  date: Date | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Presence types that open the commute step (CO2 is tied to these). */
const COMMUTE_TYPES: PresenceType[] = ["office", "remote"];

type Step = "type" | "commute";

const emptySegment = (): CommuteSegment => ({
  modeId: "",
  distanceKm: 0,
  occupants: 1,
});

export function PresenceDayDialog({
  date,
  open,
  onOpenChange,
}: PresenceDayDialogProps) {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const {
    presencesByDay,
    setPresence,
    deletePresence,
    getPresenceTrips,
    isSubmitting,
    error,
  } = usePresence();
  const { commutes, emissionFactors } = useCommute();

  const lang = i18n.resolvedLanguage ?? "fr";
  const locale = lang === "en" ? enUS : fr;
  const key = date ? dayKey(date) : null;
  const presence = key !== null ? (presencesByDay.get(key) ?? null) : null;

  const [step, setStep] = useState<Step>("type");
  const [selectedType, setSelectedType] = useState<PresenceType>("office");
  // A saved commute id, "custom" (inline trip), "none" (no trip, 0 kgCO2e), or
  // null (nothing chosen yet).
  const [choice, setChoice] = useState<string | "custom" | "none" | null>(null);
  const [customSegments, setCustomSegments] = useState<CommuteSegment[]>([
    emptySegment(),
  ]);
  const [customRoundTrip, setCustomRoundTrip] = useState(true);

  // Reset the wizard whenever the dialog (re)opens or the day changes.
  useEffect(() => {
    if (!open) return;
    setStep("type");
    setChoice(null);
    setCustomSegments([emptySegment()]);
    setCustomRoundTrip(true);
  }, [open, key]);

  // Pre-fill the custom trip from a previously-encoded office/remote day.
  useEffect(() => {
    if (!open || !presence) return;
    if (!COMMUTE_TYPES.includes(presence.type)) return;
    let cancelled = false;
    void getPresenceTrips(presence.id).then((trips) => {
      if (cancelled || trips.length === 0) return;
      setCustomSegments(
        trips.map((trip) => ({
          modeId: trip.modeId,
          distanceKm: trip.distanceKm,
          occupants: trip.occupants,
        })),
      );
      setCustomRoundTrip(trips[0]?.roundTrip ?? true);
      setChoice("custom");
    });
    return () => {
      cancelled = true;
    };
  }, [open, presence, getPresenceTrips]);

  const onSelectType = async (type: PresenceType) => {
    if (key === null) return;
    if (COMMUTE_TYPES.includes(type)) {
      setSelectedType(type);
      // Télétravail defaults to "no trip" (the usual case: working from home);
      // keep a pre-filled choice (an existing day's trips) when there is one.
      setChoice((prev) => prev ?? (type === "remote" ? "none" : null));
      setStep("commute");
      return;
    }
    // Vacation/holiday: persist immediately (no commute, 0 kgCO2e).
    const ok = await setPresence(key, type);
    if (ok) onOpenChange(false);
  };

  const buildTrips = (): PresenceTrip[] => {
    if (choice === "none") return [];
    if (choice === "custom") {
      return customSegments
        .filter((s) => s.modeId && s.distanceKm > 0)
        .map((s) => ({
          modeId: s.modeId,
          distanceKm: s.distanceKm,
          roundTrip: customRoundTrip,
          occupants: s.occupants,
        }));
    }
    const commute = commutes.find((c) => c.id === choice);
    return commute ? commuteToTrips(commute) : [];
  };

  const canConfirm =
    choice === "none"
      ? true
      : choice === "custom"
        ? customSegments.some((s) => s.modeId && s.distanceKm > 0)
        : commutes.some((c) => c.id === choice);

  const onConfirm = async () => {
    if (key === null) return;
    const ok = await setPresence(key, selectedType, buildTrips());
    if (ok) onOpenChange(false);
  };

  const onDelete = async () => {
    if (!presence) return;
    const ok = await deletePresence(presence.id);
    if (ok) onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className={cn(
          "flex max-h-[calc(100dvh-2rem)] flex-col overflow-hidden",
          step === "commute" ? "sm:max-w-md" : "sm:max-w-sm",
        )}
      >
        <DialogHeader>
          <DialogTitle className="capitalize">
            {date ? format(date, "PPPP", { locale }) : ""}
          </DialogTitle>
          <DialogDescription>
            {step === "type"
              ? t("presence.chooseType")
              : t("commute.step.title")}
          </DialogDescription>
        </DialogHeader>

        {step === "type" ? (
          <>
            <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
              <div className="grid grid-cols-2 gap-2">
                {PRESENCE_TYPES.map((type) => {
                  const active = presence?.type === type;
                  const Icon = PRESENCE_STYLES[type].Icon;
                  return (
                    <button
                      key={type}
                      type="button"
                      disabled={isSubmitting}
                      onClick={() => void onSelectType(type)}
                      className={cn(
                        "flex items-center gap-2 rounded-md border p-3 text-sm font-medium transition-colors",
                        "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                        "disabled:pointer-events-none disabled:opacity-50",
                        active
                          ? "border-ring ring-1 ring-ring"
                          : "border-border",
                      )}
                    >
                      <Icon
                        className={cn("size-4", PRESENCE_STYLES[type].text)}
                      />
                      {t(PRESENCE_STYLES[type].labelKey)}
                    </button>
                  );
                })}
              </div>

              {error && <p className="text-sm text-destructive">{error}</p>}
            </div>

            {presence && (
              <DialogFooter className="sm:justify-between">
                {COMMUTE_TYPES.includes(presence.type) && (
                  <Button
                    type="button"
                    variant="outline"
                    disabled={isSubmitting}
                    onClick={() => {
                      onOpenChange(false);
                      void navigate({
                        to: "/work-hours/$day",
                        params: { day: String(presence.day) },
                      });
                    }}
                  >
                    <Clock4 />
                    {t("workHours.encodeButton")}
                  </Button>
                )}
                <Button
                  type="button"
                  variant="destructive"
                  onClick={() => void onDelete()}
                  disabled={isSubmitting}
                >
                  {t("presence.delete")}
                </Button>
              </DialogFooter>
            )}
          </>
        ) : (
          <>
            <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
              <div className="grid gap-2">
                <button
                  type="button"
                  onClick={() => setChoice("none")}
                  className={cn(
                    "rounded-md border p-3 text-left text-sm font-medium transition-colors",
                    "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                    choice === "none"
                      ? "border-ring ring-1 ring-ring"
                      : "border-border",
                  )}
                >
                  {t("commute.step.none")}
                </button>

                {commutes.map((commute) => {
                  const active = choice === commute.id;
                  return (
                    <button
                      key={commute.id}
                      type="button"
                      onClick={() => setChoice(commute.id)}
                      className={cn(
                        "flex flex-col items-start gap-0.5 rounded-md border p-3 text-left text-sm transition-colors",
                        "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                        active
                          ? "border-ring ring-1 ring-ring"
                          : "border-border",
                      )}
                    >
                      <span className="flex w-full items-center gap-2">
                        <span className="font-medium">{commute.name}</span>
                        {typeof commute.co2Kg === "number" && (
                          <span className="ml-auto text-xs font-semibold text-primary">
                            {formatCo2(commute.co2Kg, lang)}
                          </span>
                        )}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {summarizeSegments(
                          commute.segments,
                          emissionFactors,
                          t,
                        )}
                      </span>
                    </button>
                  );
                })}

                <button
                  type="button"
                  onClick={() => setChoice("custom")}
                  className={cn(
                    "rounded-md border p-3 text-left text-sm font-medium transition-colors",
                    "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                    choice === "custom"
                      ? "border-ring ring-1 ring-ring"
                      : "border-border",
                  )}
                >
                  {t("commute.step.custom")}
                </button>
              </div>

              {choice === "custom" && (
                <div className="grid gap-3">
                  <SegmentEditor
                    segments={customSegments}
                    onChange={setCustomSegments}
                    emissionFactors={emissionFactors}
                    disabled={isSubmitting}
                  />
                  <label className="flex items-center gap-2 text-sm">
                    <input
                      type="checkbox"
                      checked={customRoundTrip}
                      onChange={(e) => setCustomRoundTrip(e.target.checked)}
                      className="size-4 rounded border-input accent-primary"
                    />
                    {t("commute.form.roundTrip")}
                  </label>
                </div>
              )}

              {error && <p className="text-sm text-destructive">{error}</p>}
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setStep("type")}
                disabled={isSubmitting}
              >
                {t("commute.step.back")}
              </Button>
              <Button
                type="button"
                onClick={() => void onConfirm()}
                disabled={isSubmitting || !canConfirm}
              >
                {t("commute.step.confirm")}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

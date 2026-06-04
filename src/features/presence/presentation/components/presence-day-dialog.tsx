import { format } from "date-fns";
import { enUS, fr } from "date-fns/locale";
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
  PRESENCE_TYPES,
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

export function PresenceDayDialog({
  date,
  open,
  onOpenChange,
}: PresenceDayDialogProps) {
  const { t, i18n } = useTranslation();
  const { presencesByDay, setPresence, deletePresence, isSubmitting, error } =
    usePresence();

  const locale = i18n.resolvedLanguage === "en" ? enUS : fr;
  const key = date ? dayKey(date) : null;
  const presence = key !== null ? (presencesByDay.get(key) ?? null) : null;

  const onSelect = async (type: PresenceType) => {
    if (key === null) return;
    const ok = await setPresence(key, type);
    if (ok) onOpenChange(false);
  };

  const onDelete = async () => {
    if (!presence) return;
    const ok = await deletePresence(presence.id);
    if (ok) onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle className="capitalize">
            {date ? format(date, "PPPP", { locale }) : ""}
          </DialogTitle>
          <DialogDescription>{t("presence.chooseType")}</DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-2 gap-2">
          {PRESENCE_TYPES.map((type) => {
            const active = presence?.type === type;
            const Icon = PRESENCE_STYLES[type].Icon;
            return (
              <button
                key={type}
                type="button"
                disabled={isSubmitting}
                onClick={() => void onSelect(type)}
                className={cn(
                  "flex items-center gap-2 rounded-md border p-3 text-sm font-medium transition-colors",
                  "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                  "disabled:pointer-events-none disabled:opacity-50",
                  active ? "border-ring ring-1 ring-ring" : "border-border",
                )}
              >
                <Icon className={cn("size-4", PRESENCE_STYLES[type].text)} />
                {t(PRESENCE_STYLES[type].labelKey)}
              </button>
            );
          })}
        </div>

        {error && <p className="text-sm text-destructive">{error}</p>}

        {presence && (
          <DialogFooter>
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
      </DialogContent>
    </Dialog>
  );
}

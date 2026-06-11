import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { ChevronLeft, Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  formatCo2,
  summarizeSegments,
} from "@/features/commute/presentation/commute-format";
import { useCommute } from "@/features/commute/presentation/hooks/use-commute";

/** "Modes de déplacement" page: lists saved commutes with links to the
 * create/edit pages. Deletion stays inline behind a confirmation dialog. */
export function CommuteListPage() {
  const { t, i18n } = useTranslation();
  const { commutes, emissionFactors, deleteCommute, isLoading } = useCommute();
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const lang = i18n.resolvedLanguage ?? "fr";

  return (
    <section className="mt-6 flex flex-1 flex-col gap-4">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("commute.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold">{t("commute.manager.title")}</h2>
        <Button className="ml-auto" asChild>
          <Link to="/commutes/new">
            <Plus />
            {t("commute.manager.add")}
          </Link>
        </Button>
      </div>
      <p className="text-sm text-muted-foreground">
        {t("commute.manager.subtitle")}
      </p>

      {isLoading ? (
        <p className="text-muted-foreground">{t("common.loading")}</p>
      ) : commutes.length === 0 ? (
        <p className="text-muted-foreground">{t("commute.manager.empty")}</p>
      ) : (
        <ul className="grid gap-2">
          {commutes.map((commute) => (
            <li
              key={commute.id}
              className="flex items-center gap-3 rounded-lg border bg-card p-3"
            >
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="font-medium">{commute.name}</span>
                  {commute.roundTrip && (
                    <span className="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                      {t("commute.manager.roundTripBadge")}
                    </span>
                  )}
                  {typeof commute.co2Kg === "number" && (
                    <span className="text-xs font-semibold text-primary">
                      {formatCo2(commute.co2Kg, lang)}
                    </span>
                  )}
                </div>
                <p className="truncate text-sm text-muted-foreground">
                  {summarizeSegments(commute.segments, emissionFactors, t)}
                </p>
              </div>
              <Button variant="ghost" size="icon-sm" asChild>
                <Link
                  to="/commutes/$commuteId/edit"
                  params={{ commuteId: commute.id }}
                  aria-label={t("commute.manager.edit")}
                >
                  <Pencil />
                </Link>
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                className="text-destructive"
                onClick={() => setConfirmId(commute.id)}
                aria-label={t("commute.manager.delete")}
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
      )}

      <Dialog
        open={confirmId !== null}
        onOpenChange={(o) => {
          if (!o) setConfirmId(null);
        }}
      >
        <DialogContent showCloseButton={false}>
          <DialogHeader>
            <DialogTitle>{t("commute.manager.deleteConfirmTitle")}</DialogTitle>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmId(null)}>
              {t("commute.form.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                const id = confirmId;
                setConfirmId(null);
                if (id) void deleteCommute(id);
              }}
            >
              {t("commute.manager.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  );
}

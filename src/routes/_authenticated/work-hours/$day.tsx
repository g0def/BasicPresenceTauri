import { Link, createFileRoute } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import { WorkHoursPage } from "@/features/work-hours/presentation/pages/work-hours-page";

export const Route = createFileRoute("/_authenticated/work-hours/$day")({
  component: WorkHoursRoute,
});

/** Work-day hours only exist on these presence types (mirrors the backend gate). */
const ELIGIBLE_TYPES = ["office", "remote"];

function WorkHoursRoute() {
  const { day } = Route.useParams();
  const { presencesByDay, isLoading } = usePresence();
  const { t } = useTranslation();

  const dayMs = Number(day);
  const presence = Number.isFinite(dayMs)
    ? (presencesByDay.get(dayMs) ?? null)
    : null;

  if (!presence || !ELIGIBLE_TYPES.includes(presence.type)) {
    // Presences load async once the active profile resolves, so a miss may
    // just mean "not loaded yet".
    if (isLoading || presencesByDay.size === 0) {
      return (
        <p className="mt-6 text-muted-foreground">{t("common.loading")}</p>
      );
    }
    return (
      <section className="mt-6 flex flex-col items-start gap-3">
        <p className="text-muted-foreground">{t("workHours.notEligible")}</p>
        <Button asChild variant="outline">
          <Link to="/">{t("workHours.back")}</Link>
        </Button>
      </section>
    );
  }

  return (
    <WorkHoursPage
      key={presence.id}
      presence={presence}
      date={new Date(dayMs)}
    />
  );
}

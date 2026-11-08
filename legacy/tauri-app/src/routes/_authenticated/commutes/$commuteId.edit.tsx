import { Link, createFileRoute, useNavigate } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { CommuteFormPage } from "@/features/commute/presentation/pages/commute-form-page";
import { useCommute } from "@/features/commute/presentation/hooks/use-commute";

export const Route = createFileRoute(
  "/_authenticated/commutes/$commuteId/edit",
)({
  component: EditCommuteRoute,
});

function EditCommuteRoute() {
  const { commuteId } = Route.useParams();
  const navigate = useNavigate();
  const { commutes, isLoading } = useCommute();
  const { t } = useTranslation();

  const commute = commutes.find((c) => c.id === commuteId);

  if (!commute) {
    // Commutes load async once the active profile resolves (isLoading starts
    // false until then), so an empty list may just mean "not loaded yet".
    if (isLoading || commutes.length === 0) {
      return (
        <p className="mt-6 text-muted-foreground">{t("common.loading")}</p>
      );
    }
    return (
      <section className="mt-6 flex flex-col items-start gap-3">
        <p className="text-muted-foreground">{t("commute.manager.notFound")}</p>
        <Button asChild variant="outline">
          <Link to="/commutes">{t("commute.step.back")}</Link>
        </Button>
      </section>
    );
  }

  return (
    <CommuteFormPage
      key={commute.id}
      commute={commute}
      onDone={() => void navigate({ to: "/commutes" })}
    />
  );
}

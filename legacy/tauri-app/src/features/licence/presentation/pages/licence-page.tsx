import { Link } from "@tanstack/react-router";
import { ChevronLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { useLicence } from "@/features/licence/presentation/hooks/use-licence";

export function LicencePage() {
  const { t } = useTranslation();
  const { html, isLoading, isError } = useLicence();

  return (
    <section className="mt-6 mx-auto w-full max-w-3xl flex flex-col gap-8 pb-12">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("licence.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold">{t("licence.title")}</h2>
      </div>

      {isLoading && (
        <p className="text-sm text-muted-foreground">{t("licence.loading")}</p>
      )}
      {isError && (
        <p className="text-sm text-destructive">{t("licence.error")}</p>
      )}
      {!isLoading && !isError && html && (
        <div
          className="prose dark:prose-invert max-w-none"
          // Rendered + sanitized backend-side (comrak unsafe_=false + ammonia).
          dangerouslySetInnerHTML={{ __html: html }}
        />
      )}
    </section>
  );
}

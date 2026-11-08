import { Link } from "@tanstack/react-router";
import { ChevronLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { useProfile } from "@/entities/profile/model/use-profile";
import { Co2Section } from "@/pages/settings/sections/co2-section";
import {
  ExportSection,
  TransferSection,
} from "@/pages/settings/sections/data-sections";
import { PreferencesSection } from "@/pages/settings/sections/preferences-section";
import { ProfilesSection } from "@/pages/settings/sections/profiles-section";
import { SecuritySection } from "@/pages/settings/sections/security-section";

/** Settings screen: composes the sections owned by the various features. */
export function SettingsPage() {
  const { t } = useTranslation();
  const { activeProfile } = useProfile();

  return (
    <section className="mt-6 mx-auto w-full max-w-2xl flex flex-col gap-8">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("settings.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold">{t("settings.title")}</h2>
      </div>

      <ProfilesSection />

      {/* Per-profile preferences — need an active profile to read/write into. */}
      {activeProfile && (
        <>
          <PreferencesSection />
          <Co2Section />
          <ExportSection profileId={activeProfile.id} />
          <TransferSection profileId={activeProfile.id} />
        </>
      )}

      <SecuritySection />
    </section>
  );
}

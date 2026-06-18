import { createFileRoute } from "@tanstack/react-router";

import { SettingsPage } from "@/features/settings/presentation/pages/settings-page";

export const Route = createFileRoute("/_authenticated/settings")({
  component: SettingsPage,
});

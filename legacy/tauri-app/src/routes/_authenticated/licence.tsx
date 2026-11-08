import { createFileRoute } from "@tanstack/react-router";

import { LicencePage } from "@/features/licence/presentation/pages/licence-page";

export const Route = createFileRoute("/_authenticated/licence")({
  component: LicencePage,
});

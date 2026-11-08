import { createFileRoute } from "@tanstack/react-router";

import { MethodologyPage } from "@/features/methodology/presentation/pages/methodology-page";

export const Route = createFileRoute("/_authenticated/methodology")({
  component: MethodologyPage,
});

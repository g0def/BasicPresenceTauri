import { createFileRoute } from "@tanstack/react-router";

import { CommuteListPage } from "@/features/commute/presentation/pages/commute-list-page";

export const Route = createFileRoute("/_authenticated/commutes/")({
  component: CommuteListPage,
});

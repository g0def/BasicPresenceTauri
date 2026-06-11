import { createFileRoute } from "@tanstack/react-router";

import { PresenceCalendar } from "@/features/presence/presentation/components/presence-calendar";

export const Route = createFileRoute("/_authenticated/")({
  component: PresenceCalendar,
});

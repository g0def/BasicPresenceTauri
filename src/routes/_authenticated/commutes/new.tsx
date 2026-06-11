import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { CommuteFormPage } from "@/features/commute/presentation/pages/commute-form-page";

export const Route = createFileRoute("/_authenticated/commutes/new")({
  component: NewCommuteRoute,
});

function NewCommuteRoute() {
  const navigate = useNavigate();
  return <CommuteFormPage onDone={() => void navigate({ to: "/commutes" })} />;
}

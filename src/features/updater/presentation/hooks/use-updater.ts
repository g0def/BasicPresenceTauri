import { useContext } from "react";

import { UpdaterContext } from "@/features/updater/presentation/context/updater-context";

export function useUpdater() {
  const ctx = useContext(UpdaterContext);
  if (!ctx) throw new Error("useUpdater must be used within UpdaterProvider");
  return ctx;
}

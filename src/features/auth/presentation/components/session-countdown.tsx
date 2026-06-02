import { useTranslation } from "react-i18next";

import { COUNTDOWN_WARNING_MS } from "@/core/config";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";

function formatRemaining(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function SessionCountdown() {
  const { remainingMs } = useAuth();
  const { t } = useTranslation();
  const isWarning = remainingMs <= COUNTDOWN_WARNING_MS;

  return (
    <span
      className={isWarning ? "countdown countdown--warning" : "countdown"}
      title={t("session.tooltip")}
    >
      {t("session.label", { time: formatRemaining(remainingMs) })}
    </span>
  );
}

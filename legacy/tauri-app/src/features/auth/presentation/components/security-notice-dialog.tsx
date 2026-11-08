import { ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";

/**
 * Shown once after login when something security-relevant happened since the
 * previous one (failed attempts, lockout, unclean shutdown, password change),
 * so the owner notices activity they did not cause.
 */
export function SecurityNoticeDialog() {
  const { t, i18n } = useTranslation();
  const { securityNotice, dismissSecurityNotice } = useAuth();
  if (!securityNotice) return null;

  const formatter = new Intl.DateTimeFormat(i18n.resolvedLanguage ?? "fr", {
    dateStyle: "medium",
    timeStyle: "short",
  });

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) dismissSecurityNotice();
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ShieldAlert className="size-5 text-warning" aria-hidden="true" />
            {t("auth.securityNotice.title")}
          </DialogTitle>
          <DialogDescription asChild>
            <ul className="grid gap-1.5 text-sm">
              {securityNotice.failedAttempts > 0 && (
                <li>
                  {t("auth.securityNotice.failedAttempts", {
                    count: securityNotice.failedAttempts,
                  })}
                </li>
              )}
              {securityNotice.events.map((event, i) => (
                <li key={`${event.kind}-${event.at}-${i}`}>
                  <span className="tabular-nums text-muted-foreground">
                    {formatter.format(event.at)}
                  </span>{" "}
                  —{" "}
                  {t(`auth.securityNotice.kinds.${event.kind}`, {
                    defaultValue: event.kind,
                  })}
                </li>
              ))}
            </ul>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={dismissSecurityNotice}>
            {t("auth.securityNotice.dismiss")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

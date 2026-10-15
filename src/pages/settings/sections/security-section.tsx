import { useTranslation } from "react-i18next";

import { ChangePasswordForm } from "@/features/auth/presentation/components/change-password-form";

/** Account security: password change, and the no-recovery reminder. */
export function SecuritySection() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-3">
      <h3 className="text-base font-medium">{t("settings.securitySection")}</h3>
      <p className="text-sm text-muted-foreground">
        {t("auth.noRecoveryWarning")}
      </p>
      <ChangePasswordForm />
    </div>
  );
}

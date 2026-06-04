import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";

import profilePic from "@/assets/picture/minimal-profile-accout.png";
import { Button } from "@/components/ui/button";
import { ProfileFormDialog } from "@/features/profile/presentation/components/profile-form-dialog";

/** Shown after login when the account has no profile yet. */
export function EmptyProfileState() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  return (
    <section className="mt-24 flex flex-col items-center gap-6 text-center">
      <img
        src={profilePic}
        alt=""
        aria-hidden
        className="size-24 rounded-full opacity-70"
      />
      <div className="space-y-1">
        <h2 className="text-xl font-semibold">{t("profile.emptyTitle")}</h2>
        <p className="text-muted-foreground">{t("profile.emptySubtitle")}</p>
      </div>
      <Button size="lg" onClick={() => setOpen(true)}>
        <Plus />
        {t("profile.add")}
      </Button>
      <ProfileFormDialog open={open} onOpenChange={setOpen} />
    </section>
  );
}

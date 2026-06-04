import { useEffect, useState } from "react";
import type { FormEvent } from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { Profile } from "@/features/profile/domain/entities/profile";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

interface ProfileFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** When provided, the dialog edits this profile; otherwise it creates one. */
  profile?: Profile | null;
}

export function ProfileFormDialog({
  open,
  onOpenChange,
  profile,
}: ProfileFormDialogProps) {
  const { t } = useTranslation();
  const { createProfile, updateProfile, isSubmitting, error, clearError } =
    useProfile();
  const isEdit = Boolean(profile);

  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [enterprise, setEnterprise] = useState("");
  const [poste, setPoste] = useState("");

  // Reset the fields whenever the dialog (re)opens or the edited profile changes.
  useEffect(() => {
    if (!open) return;
    setFirstName(profile?.firstName ?? "");
    setLastName(profile?.lastName ?? "");
    setEnterprise(profile?.enterprise ?? "");
    setPoste(profile?.poste ?? "");
    clearError();
  }, [open, profile, clearError]);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const fields = {
      firstName,
      lastName,
      enterprise,
      poste: poste.trim() ? poste : null,
    };
    const ok =
      isEdit && profile
        ? await updateProfile({ id: profile.id, ...fields })
        : await createProfile(fields);
    if (ok) onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form onSubmit={onSubmit} className="grid gap-4">
          <DialogHeader>
            <DialogTitle>
              {isEdit ? t("profile.editTitle") : t("profile.addTitle")}
            </DialogTitle>
            <DialogDescription>{t("profile.formSubtitle")}</DialogDescription>
          </DialogHeader>

          <div className="grid gap-2">
            <Label htmlFor="profile-first-name">{t("profile.firstName")}</Label>
            <Input
              id="profile-first-name"
              value={firstName}
              onChange={(e) => setFirstName(e.target.value)}
              required
              autoFocus
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="profile-last-name">{t("profile.lastName")}</Label>
            <Input
              id="profile-last-name"
              value={lastName}
              onChange={(e) => setLastName(e.target.value)}
              required
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="profile-enterprise">
              {t("profile.enterprise")}
            </Label>
            <Input
              id="profile-enterprise"
              value={enterprise}
              onChange={(e) => setEnterprise(e.target.value)}
              required
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="profile-poste">
              {t("profile.poste")}{" "}
              <span className="text-muted-foreground">
                {t("profile.optional")}
              </span>
            </Label>
            <Input
              id="profile-poste"
              value={poste}
              onChange={(e) => setPoste(e.target.value)}
            />
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              {t("profile.cancel")}
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? t("profile.saving") : t("profile.save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

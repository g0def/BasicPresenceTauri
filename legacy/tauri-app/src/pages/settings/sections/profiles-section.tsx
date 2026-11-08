import { useState } from "react";
import { Pencil, Plus, Trash2 } from "lucide-react";
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
import { fullName } from "@/entities/profile/model/profile";
import type { Profile } from "@/entities/profile/model/profile";
import { useProfile } from "@/entities/profile/model/use-profile";
import { ProfileFormDialog } from "@/features/profile/presentation/components/profile-form-dialog";

/** List of the owner's profiles, with create / edit / delete (confirmed). */
export function ProfilesSection() {
  const { t } = useTranslation();
  const { profiles, activeProfile, deleteProfile } = useProfile();
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [confirmId, setConfirmId] = useState<string | null>(null);

  const openCreate = () => {
    setEditing(null);
    setFormOpen(true);
  };
  const openEdit = (profile: Profile) => {
    setEditing(profile);
    setFormOpen(true);
  };

  const profileToDelete = confirmId
    ? (profiles.find((p) => p.id === confirmId) ?? null)
    : null;

  return (
    <>
      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-medium">
            {t("settings.profilesSection")}
          </h3>
          <Button size="sm" onClick={openCreate}>
            <Plus />
            {t("profile.add")}
          </Button>
        </div>

        {profiles.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {t("profile.emptySubtitle")}
          </p>
        ) : (
          <ul className="grid gap-2">
            {profiles.map((profile) => (
              <li
                key={profile.id}
                className="flex items-center gap-3 rounded-lg border bg-card p-3"
              >
                <div className="min-w-0 flex-1">
                  <span className="font-medium">{fullName(profile)}</span>
                  {profile.enterprise && (
                    <p className="truncate text-sm text-muted-foreground">
                      {profile.enterprise}
                      {profile.poste ? ` — ${profile.poste}` : ""}
                    </p>
                  )}
                </div>
                {activeProfile?.id === profile.id && (
                  <span className="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                    {t("profile.active")}
                  </span>
                )}
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => openEdit(profile)}
                  aria-label={t("profile.edit")}
                >
                  <Pencil />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  className="text-destructive"
                  onClick={() => setConfirmId(profile.id)}
                  aria-label={t("profile.delete")}
                >
                  <Trash2 />
                </Button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <ProfileFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        profile={editing}
      />

      <Dialog
        open={confirmId !== null}
        onOpenChange={(o) => {
          if (!o) setConfirmId(null);
        }}
      >
        <DialogContent showCloseButton={false}>
          <DialogHeader>
            <DialogTitle>{t("profile.deleteConfirmTitle")}</DialogTitle>
            <DialogDescription>
              {t("profile.deleteConfirmBody", {
                name: profileToDelete ? fullName(profileToDelete) : "",
              })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmId(null)}>
              {t("profile.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                const id = confirmId;
                setConfirmId(null);
                if (id) void deleteProfile(id);
              }}
            >
              {t("profile.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

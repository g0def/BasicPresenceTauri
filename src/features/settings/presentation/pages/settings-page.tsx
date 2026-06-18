import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { ChevronLeft, Pencil, Plus, Trash2 } from "lucide-react";
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
import { fullName } from "@/features/profile/domain/entities/profile";
import type { Profile } from "@/features/profile/domain/entities/profile";
import { ProfileFormDialog } from "@/features/profile/presentation/components/profile-form-dialog";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { ImportDialog } from "@/features/presence/presentation/components/import-dialog";
import { NOTE_FONTS, useNoteFont } from "@/shared/note-font/use-note-font";

export function SettingsPage() {
  const { t } = useTranslation();
  const { profiles, activeProfile, deleteProfile } = useProfile();
  const { noteFont, setNoteFont } = useNoteFont();

  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [importOpen, setImportOpen] = useState(false);

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
    <section className="mt-6 mx-auto w-full max-w-2xl flex flex-col gap-8">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("settings.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold">{t("settings.title")}</h2>
      </div>

      {/* Profiles section */}
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

      {/* Note font section */}
      <div className="flex flex-col gap-3">
        <h3 className="text-base font-medium">
          {t("settings.noteFontSection")}
        </h3>
        <div className="flex flex-wrap gap-2">
          {NOTE_FONTS.map((font) => (
            <button
              key={font}
              type="button"
              onClick={() => setNoteFont(font)}
              className={`flex items-center gap-3 rounded-md border px-4 py-2 text-sm transition-colors ${
                noteFont === font
                  ? "border-primary bg-primary text-primary-foreground"
                  : "bg-card hover:bg-accent"
              }`}
            >
              <span>{t(`noteFont.${font}`)}</span>
              {/* Preview resolves the same --note-font the notes use (index.css),
                  so the sample never drifts from the real applied font. */}
              <span
                data-note-font={font}
                className="opacity-60"
                style={{ fontFamily: "var(--note-font)" }}
              >
                Aa
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Import section — needs an active profile to import into; without one
          importPresences silently no-ops (presence-provider), so hide it. */}
      {activeProfile && (
        <div className="flex flex-col gap-3">
          <h3 className="text-base font-medium">{t("import.title")}</h3>
          <p className="text-sm text-muted-foreground">
            {t("import.description")}
          </p>
          <div>
            <Button variant="outline" onClick={() => setImportOpen(true)}>
              {t("import.menuItem")}
            </Button>
          </div>
        </div>
      )}

      <ProfileFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        profile={editing}
      />

      <ImportDialog open={importOpen} onOpenChange={setImportOpen} />

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
    </section>
  );
}

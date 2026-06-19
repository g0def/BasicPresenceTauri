import { useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { ChevronLeft, Download, Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useProfileSettings } from "@/features/profile-settings/presentation/hooks/use-profile-settings";
import { fullName } from "@/features/profile/domain/entities/profile";
import type { Profile } from "@/features/profile/domain/entities/profile";
import { ProfileFormDialog } from "@/features/profile/presentation/components/profile-form-dialog";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { ExportDialog } from "@/features/export/presentation/components/export-dialog";
import { ExportProfileDialog } from "@/features/profile-transfer/presentation/components/export-profile-dialog";
import { ImportProfileDialog } from "@/features/profile-transfer/presentation/components/import-profile-dialog";
import { StartTimePicker } from "@/features/work-hours/presentation/components/start-time-picker";
import type { CellDisplayMode } from "@/shared/cell-display/cell-display-context";
import { NOTE_FONTS } from "@/shared/note-font/use-note-font";

const CELL_DISPLAY_MODES: CellDisplayMode[] = ["co2", "hours"];
const GRID_COUNTRIES = ["FR", "BE", "DE", "EU"] as const;

export function SettingsPage() {
  const { t } = useTranslation();
  const { profiles, activeProfile, deleteProfile } = useProfile();
  const { settings, update } = useProfileSettings();

  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [exportOpen, setExportOpen] = useState(false);
  const [exportBundleOpen, setExportBundleOpen] = useState(false);
  const [importBundleOpen, setImportBundleOpen] = useState(false);

  // Car-occupancy is a free number field: keep a local draft, commit on blur.
  const [occupancy, setOccupancy] = useState(
    String(settings.co2.defaultCarOccupancy),
  );
  useEffect(() => {
    setOccupancy(String(settings.co2.defaultCarOccupancy));
  }, [settings.co2.defaultCarOccupancy]);
  const commitOccupancy = () => {
    const n = Math.round(Number(occupancy));
    if (
      Number.isFinite(n) &&
      n >= 1 &&
      n !== settings.co2.defaultCarOccupancy
    ) {
      void update({ co2: { defaultCarOccupancy: n } });
    } else {
      setOccupancy(String(settings.co2.defaultCarOccupancy));
    }
  };

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

      {/* Per-profile preferences — need an active profile to read/write into. */}
      {activeProfile && (
        <>
          {/* Default log hour */}
          <div className="flex flex-col gap-3">
            <h3 className="text-base font-medium">
              {t("settings.logHourSection")}
            </h3>
            <p className="text-sm text-muted-foreground">
              {t("settings.defaultStartTimeHint")}
            </p>
            <StartTimePicker
              id="default-start-time"
              value={settings.startMinutes}
              onCommit={(startMinutes) => void update({ startMinutes })}
              label={t("settings.defaultStartTime")}
            />
          </div>

          {/* Note font */}
          <div className="flex flex-col gap-3">
            <h3 className="text-base font-medium">
              {t("settings.noteFontSection")}
            </h3>
            <div className="flex flex-wrap gap-2">
              {NOTE_FONTS.map((font) => (
                <button
                  key={font}
                  type="button"
                  onClick={() => void update({ noteFont: font })}
                  className={`flex items-center gap-3 rounded-md border px-4 py-2 text-sm transition-colors ${
                    settings.noteFont === font
                      ? "border-primary bg-primary text-primary-foreground"
                      : "bg-card hover:bg-accent"
                  }`}
                >
                  <span>{t(`noteFont.${font}`)}</span>
                  {/* Preview resolves the same --note-font the notes use
                      (index.css), so the sample never drifts from the real font. */}
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

          {/* Calendar cell display */}
          <div className="flex flex-col gap-3">
            <h3 className="text-base font-medium">
              {t("settings.cellDisplaySection")}
            </h3>
            <div className="flex flex-wrap gap-2">
              {CELL_DISPLAY_MODES.map((mode) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => void update({ cellDisplayMode: mode })}
                  className={`rounded-md border px-4 py-2 text-sm transition-colors ${
                    settings.cellDisplayMode === mode
                      ? "border-primary bg-primary text-primary-foreground"
                      : "bg-card hover:bg-accent"
                  }`}
                >
                  {t(`cellDisplay.${mode}`)}
                </button>
              ))}
            </div>
          </div>

          {/* CO2 configuration */}
          <div className="flex flex-col gap-4">
            <h3 className="text-base font-medium">
              {t("settings.co2Section")}
            </h3>

            <div className="grid gap-1.5">
              <span className="text-xs text-muted-foreground">
                {t("settings.co2.gridCountry")}
              </span>
              <div className="flex flex-wrap gap-2">
                {GRID_COUNTRIES.map((country) => (
                  <button
                    key={country}
                    type="button"
                    onClick={() =>
                      void update({ co2: { gridCountry: country } })
                    }
                    className={`rounded-md border px-4 py-2 text-sm transition-colors ${
                      settings.co2.gridCountry === country
                        ? "border-primary bg-primary text-primary-foreground"
                        : "bg-card hover:bg-accent"
                    }`}
                  >
                    {t(`settings.co2.countries.${country}`)}
                  </button>
                ))}
              </div>
            </div>

            <div className="grid gap-1.5">
              <Label
                htmlFor="car-occupancy"
                className="text-xs text-muted-foreground"
              >
                {t("settings.co2.carOccupancy")}
              </Label>
              <Input
                id="car-occupancy"
                type="number"
                min={1}
                className="w-24 tabular-nums"
                value={occupancy}
                onChange={(e) => setOccupancy(e.target.value)}
                onBlur={commitOccupancy}
                onKeyDown={(e) => {
                  if (e.key === "Enter") e.currentTarget.blur();
                }}
              />
            </div>

            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                className="size-4"
                checked={settings.co2.includeRadiativeForcing}
                onChange={(e) =>
                  void update({
                    co2: { includeRadiativeForcing: e.target.checked },
                  })
                }
              />
              {t("settings.co2.radiativeForcing")}
            </label>
          </div>
        </>
      )}

      {/* Spreadsheet export (.ods) — needs an active profile to know whose data
          to export. */}
      {activeProfile && (
        <div className="flex flex-col gap-3">
          <h3 className="text-base font-medium">{t("export.title")}</h3>
          <p className="text-sm text-muted-foreground">
            {t("export.description")}
          </p>
          <div>
            <Button onClick={() => setExportOpen(true)}>
              <Download />
              {t("export.menuItem")}
            </Button>
          </div>
        </div>
      )}

      {/* Profile transfer (shareable .json): export the whole profile, or import
          one (into a new profile or merged into an existing one). */}
      {activeProfile && (
        <div className="flex flex-col gap-3">
          <h3 className="text-base font-medium">{t("transfer.section")}</h3>
          <p className="text-sm text-muted-foreground">
            {t("transfer.sectionHint")}
          </p>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => setExportBundleOpen(true)}>
              {t("transfer.export.menuItem")}
            </Button>
            <Button variant="outline" onClick={() => setImportBundleOpen(true)}>
              {t("transfer.import.menuItem")}
            </Button>
          </div>
        </div>
      )}

      <ProfileFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        profile={editing}
      />

      {activeProfile && (
        <ExportDialog
          open={exportOpen}
          onOpenChange={setExportOpen}
          profileId={activeProfile.id}
        />
      )}

      {activeProfile && (
        <ExportProfileDialog
          open={exportBundleOpen}
          onOpenChange={setExportBundleOpen}
          profileId={activeProfile.id}
        />
      )}

      <ImportProfileDialog
        open={importBundleOpen}
        onOpenChange={setImportBundleOpen}
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
    </section>
  );
}

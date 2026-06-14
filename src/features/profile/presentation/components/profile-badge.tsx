import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  BellRing,
  ChevronDown,
  DownloadCloud,
  LogOut,
  Moon,
  Pencil,
  Plus,
  Route,
  Sun,
  Trash2,
  Upload,
} from "lucide-react";

import profilePic from "@/assets/picture/minimal-profile-accout.png";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { supportedLngs } from "@/core/i18n/resources";
import { ImportDialog } from "@/features/presence/presentation/components/import-dialog";
import type { Profile } from "@/features/profile/domain/entities/profile";
import { ProfileFormDialog } from "@/features/profile/presentation/components/profile-form-dialog";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { useUpdater } from "@/features/updater/presentation/hooks/use-updater";
import {
  useCellDisplayMode,
  type CellDisplayMode,
} from "@/shared/cell-display/use-cell-display-mode";
import { useNoteFont, type NoteFont } from "@/shared/note-font/use-note-font";
import { useTheme } from "@/shared/theme/use-theme";

function fullName(p: Profile): string {
  return `${p.firstName} ${p.lastName}`;
}

interface ProfileBadgeProps {
  /** Auth concern, injected by the composition root (App/Home) to keep the
   * profile feature decoupled from the auth feature. */
  onLogout: () => void;
  /** Open the "Modes de déplacement" view (owned by Home). */
  onOpenCommutes: () => void;
  /** Trigger a manual update check (owned by the updater feature). */
  onCheckUpdates: () => void;
}

/**
 * Header account menu: active-profile photo + name on the right, with a menu to
 * switch/add/edit/delete profiles and to manage language, theme and logout.
 */
export function ProfileBadge({
  onLogout,
  onOpenCommutes,
  onCheckUpdates,
}: ProfileBadgeProps) {
  const { t, i18n } = useTranslation();
  const { profiles, activeProfile, setActiveProfile, deleteProfile } =
    useProfile();
  const { theme, toggleTheme } = useTheme();
  const { autoUpdateEnabled, toggleAutoUpdate } = useUpdater();
  const { mode: cellDisplayMode, setMode: setCellDisplayMode } =
    useCellDisplayMode();
  const { noteFont, setNoteFont } = useNoteFont();
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);

  const openCreate = () => {
    setEditing(null);
    setFormOpen(true);
  };
  const openEdit = () => {
    setEditing(activeProfile);
    setFormOpen(true);
  };

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button
            type="button"
            aria-label={t("profile.menu")}
            className="flex items-center gap-2 rounded-full border bg-background py-1 pr-2 pl-1 text-sm shadow-xs transition-colors hover:bg-accent focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
          >
            <img src={profilePic} alt="" className="size-7 rounded-full" />
            {activeProfile && (
              <span className="font-medium">{fullName(activeProfile)}</span>
            )}
            <ChevronDown className="size-4 opacity-60" />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-56">
          {profiles.length > 0 && (
            <>
              <DropdownMenuLabel>
                {t("profile.profilesLabel")}
              </DropdownMenuLabel>
              <DropdownMenuRadioGroup
                value={activeProfile?.id ?? ""}
                onValueChange={(id) => void setActiveProfile(id)}
              >
                {profiles.map((p) => (
                  <DropdownMenuRadioItem key={p.id} value={p.id}>
                    {fullName(p)}
                  </DropdownMenuRadioItem>
                ))}
              </DropdownMenuRadioGroup>
              <DropdownMenuSeparator />
            </>
          )}

          <DropdownMenuItem onSelect={openCreate}>
            <Plus />
            {t("profile.add")}
          </DropdownMenuItem>
          {activeProfile && (
            <>
              <DropdownMenuItem onSelect={openEdit}>
                <Pencil />
                {t("profile.edit")}
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={() => onOpenCommutes()}>
                <Route />
                {t("commute.menuItem")}
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={() => setImportOpen(true)}>
                <Upload />
                {t("import.menuItem")}
              </DropdownMenuItem>
              <DropdownMenuItem
                variant="destructive"
                onSelect={() => setConfirmOpen(true)}
              >
                <Trash2 />
                {t("profile.delete")}
              </DropdownMenuItem>
            </>
          )}

          <DropdownMenuSeparator />
          <DropdownMenuLabel>{t("language.label")}</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            value={i18n.resolvedLanguage ?? "fr"}
            onValueChange={(value) => void i18n.changeLanguage(value)}
          >
            {supportedLngs.map((lng) => (
              <DropdownMenuRadioItem key={lng} value={lng}>
                {t(`language.${lng}`)}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>

          <DropdownMenuSeparator />
          <DropdownMenuLabel>{t("cellDisplay.label")}</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            value={cellDisplayMode}
            onValueChange={(v) => setCellDisplayMode(v as CellDisplayMode)}
          >
            <DropdownMenuRadioItem value="co2">
              {t("cellDisplay.co2")}
            </DropdownMenuRadioItem>
            <DropdownMenuRadioItem value="hours">
              {t("cellDisplay.hours")}
            </DropdownMenuRadioItem>
          </DropdownMenuRadioGroup>

          <DropdownMenuSeparator />
          <DropdownMenuLabel>{t("noteFont.label")}</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            value={noteFont}
            onValueChange={(v) => setNoteFont(v as NoteFont)}
          >
            <DropdownMenuRadioItem value="sans">
              {t("noteFont.sans")}
            </DropdownMenuRadioItem>
            <DropdownMenuRadioItem value="serif">
              {t("noteFont.serif")}
            </DropdownMenuRadioItem>
            <DropdownMenuRadioItem value="mono">
              {t("noteFont.mono")}
            </DropdownMenuRadioItem>
          </DropdownMenuRadioGroup>

          <DropdownMenuSeparator />
          {/* Keep the menu open on toggle so it reads like a switch. */}
          <DropdownMenuItem
            onSelect={(e) => {
              e.preventDefault();
              toggleTheme();
            }}
          >
            {theme === "dark" ? <Sun /> : <Moon />}
            {theme === "dark" ? t("theme.light") : t("theme.dark")}
          </DropdownMenuItem>

          <DropdownMenuSeparator />
          <DropdownMenuItem onSelect={() => onCheckUpdates()}>
            <DownloadCloud />
            {t("updater.menuItem")}
          </DropdownMenuItem>
          {/* Keep the menu open on toggle so it reads like a switch. */}
          <DropdownMenuItem
            onSelect={(e) => {
              e.preventDefault();
              toggleAutoUpdate();
            }}
          >
            <BellRing />
            {t("updater.autoUpdate")}
            <span className="ml-auto text-xs text-muted-foreground">
              {autoUpdateEnabled ? t("updater.on") : t("updater.off")}
            </span>
          </DropdownMenuItem>

          <DropdownMenuSeparator />
          <DropdownMenuItem variant="destructive" onSelect={() => onLogout()}>
            <LogOut />
            {t("home.logout")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <ProfileFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        profile={editing}
      />

      <ImportDialog open={importOpen} onOpenChange={setImportOpen} />

      {activeProfile && (
        <Dialog open={confirmOpen} onOpenChange={setConfirmOpen}>
          <DialogContent showCloseButton={false}>
            <DialogHeader>
              <DialogTitle>{t("profile.deleteConfirmTitle")}</DialogTitle>
              <DialogDescription>
                {t("profile.deleteConfirmBody", {
                  name: fullName(activeProfile),
                })}
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setConfirmOpen(false)}>
                {t("profile.cancel")}
              </Button>
              <Button
                variant="destructive"
                onClick={() => {
                  const id = activeProfile.id;
                  setConfirmOpen(false);
                  void deleteProfile(id);
                }}
              >
                {t("profile.delete")}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </>
  );
}

import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { ChevronDown, LogOut, Moon, Route, Settings, Sun } from "lucide-react";

import profilePic from "@/assets/picture/minimal-profile-accout.png";
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
import { fullName } from "@/features/profile/domain/entities/profile";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import {
  useCellDisplayMode,
  type CellDisplayMode,
} from "@/shared/cell-display/use-cell-display-mode";
import { useTheme } from "@/shared/theme/use-theme";

interface ProfileBadgeProps {
  /** Auth concern, injected by the composition root (App/Home) to keep the
   * profile feature decoupled from the auth feature. */
  onLogout: () => void;
  /** Open the "Modes de déplacement" view (owned by Home). */
  onOpenCommutes: () => void;
}

/**
 * Header account menu: active-profile photo + name on the right, with a menu to
 * switch the active profile, set language/calendar-display/theme, open the
 * settings page and log out. Profile CRUD and import live on the settings page.
 */
export function ProfileBadge({ onLogout, onOpenCommutes }: ProfileBadgeProps) {
  const { t, i18n } = useTranslation();
  const { profiles, activeProfile, setActiveProfile } = useProfile();
  const { theme, toggleTheme } = useTheme();
  const { mode: cellDisplayMode, setMode: setCellDisplayMode } =
    useCellDisplayMode();
  return (
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
            <DropdownMenuLabel>{t("profile.profilesLabel")}</DropdownMenuLabel>
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

        {activeProfile && (
          <DropdownMenuItem onSelect={() => onOpenCommutes()}>
            <Route />
            {t("commute.menuItem")}
          </DropdownMenuItem>
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
        <DropdownMenuItem asChild>
          <Link to="/settings">
            <Settings />
            {t("settings.menuItem")}
          </Link>
        </DropdownMenuItem>

        {/* Updates are handled from the header version badge: the check runs
              automatically and installing is a manual click there. */}
        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" onSelect={() => onLogout()}>
          <LogOut />
          {t("home.logout")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

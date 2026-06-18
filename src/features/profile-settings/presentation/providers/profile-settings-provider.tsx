import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { isAppError } from "@/core/errors";
import { TauriProfileSettingsRepository } from "@/features/profile-settings/data/repositories/tauri-profile-settings.repository";
import {
  DEFAULT_PROFILE_SETTINGS,
  type ProfileSettings,
  type ProfileSettingsInput,
} from "@/features/profile-settings/domain/entities/profile-settings";
import { makeGetProfileSettingsUseCase } from "@/features/profile-settings/domain/use-cases/get-profile-settings";
import { makeSetProfileSettingsUseCase } from "@/features/profile-settings/domain/use-cases/set-profile-settings";
import {
  ProfileSettingsContext,
  type ProfileSettingsContextValue,
} from "@/features/profile-settings/presentation/context/profile-settings-context";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import type { CellDisplayMode } from "@/shared/cell-display/cell-display-context";
import {
  applyNoteFont,
  NOTE_FONTS,
  type NoteFont,
} from "@/shared/note-font/use-note-font";

// Composition root for the profile-settings feature.
const repo = new TauriProfileSettingsRepository();
const getUseCase = makeGetProfileSettingsUseCase(repo);
const setUseCase = makeSetProfileSettingsUseCase(repo);

// One-time, device-level migration: before this feature, note-font and the
// calendar cell-display mode lived in device-global localStorage. Fold whatever
// the user had chosen into the first active profile so the preference isn't lost
// when these settings become per-profile. Guarded by a flag so it runs once.
const LEGACY_MIGRATED_KEY = "profile-settings-legacy-migrated";

type LegacyPrefs = Partial<
  Pick<ProfileSettings, "noteFont" | "cellDisplayMode">
>;

function readLegacyPrefs(): LegacyPrefs | null {
  if (typeof localStorage === "undefined") return null;
  if (localStorage.getItem(LEGACY_MIGRATED_KEY)) return null;
  const out: LegacyPrefs = {};
  const font = localStorage.getItem("note-font");
  if (font && NOTE_FONTS.includes(font as NoteFont))
    out.noteFont = font as NoteFont;
  const cell = localStorage.getItem("cell-display-mode");
  if (cell === "co2" || cell === "hours")
    out.cellDisplayMode = cell as CellDisplayMode;
  return Object.keys(out).length > 0 ? out : null;
}

interface ProfileSettingsProviderProps {
  children: ReactNode;
  /** Called when a command reports a locked vault (SESSION_EXPIRED). Injected
   * by the composition root so the feature stays independent of auth. */
  onSessionExpired?: () => void;
}

export function ProfileSettingsProvider({
  children,
  onSessionExpired,
}: ProfileSettingsProviderProps) {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const [settings, setSettings] = useState<ProfileSettings>(
    DEFAULT_PROFILE_SETTINGS,
  );
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleError = useCallback(
    (e: unknown) => {
      if (isAppError(e) && e.code === "SESSION_EXPIRED") {
        onSessionExpired?.();
        return;
      }
      setError(isAppError(e) ? e.message : "Une erreur est survenue");
    },
    [onSessionExpired],
  );

  // Load (and reset on) active-profile change. Without a profile there is
  // nothing to load — fall back to defaults.
  useEffect(() => {
    if (!profileId) {
      setSettings(DEFAULT_PROFILE_SETTINGS);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    getUseCase(profileId)
      .then((loaded) => {
        if (cancelled) return;
        const legacy = readLegacyPrefs();
        if (legacy) {
          // Fold the legacy device prefs into this profile, once.
          const merged: ProfileSettings = { ...loaded, ...legacy };
          setSettings(merged);
          void setUseCase(profileId, merged).catch(() => {});
        } else {
          setSettings(loaded);
        }
        if (typeof localStorage !== "undefined") {
          localStorage.setItem(LEGACY_MIGRATED_KEY, "1");
        }
      })
      .catch((e) => {
        if (!cancelled) handleError(e);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [profileId, handleError]);

  // Authoritative note-font reconcile: apply the active profile's font (and
  // refresh the pre-React paint cache). With no profile, restore the default.
  useEffect(() => {
    applyNoteFont(
      profileId ? settings.noteFont : DEFAULT_PROFILE_SETTINGS.noteFont,
    );
  }, [profileId, settings.noteFont]);

  const update = useCallback(
    async (input: ProfileSettingsInput): Promise<boolean> => {
      if (!profileId) return false;
      const previous = settings;
      const optimistic: ProfileSettings = {
        ...settings,
        ...input,
        co2: { ...settings.co2, ...(input.co2 ?? {}) },
      };
      setSettings(optimistic); // optimistic; the note-font effect fires now
      setIsSubmitting(true);
      setError(null);
      try {
        const canonical = await setUseCase(profileId, optimistic);
        setSettings(canonical);
        return true;
      } catch (e) {
        setSettings(previous); // roll back
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [profileId, settings, handleError],
  );

  const clearError = useCallback(() => setError(null), []);

  const value = useMemo<ProfileSettingsContextValue>(
    () => ({
      settings,
      isLoading,
      isSubmitting,
      error,
      update,
      clearError,
    }),
    [settings, isLoading, isSubmitting, error, update, clearError],
  );

  return (
    <ProfileSettingsContext.Provider value={value}>
      {children}
    </ProfileSettingsContext.Provider>
  );
}

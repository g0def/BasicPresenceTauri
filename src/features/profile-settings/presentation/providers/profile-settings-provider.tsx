import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
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
import { useProfile } from "@/entities/profile/model/use-profile";
import {
  applyNoteFont,
  NOTE_FONTS,
  type NoteFont,
} from "@/shared/note-font/use-note-font";

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
  if (cell === "co2" || cell === "hours") out.cellDisplayMode = cell;
  return Object.keys(out).length > 0 ? out : null;
}

/** TanStack Query key of a profile's settings row. */
const settingsKey = (profileId: string | null) =>
  ["profile-settings", profileId] as const;

export function ProfileSettingsProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const { profileSettingsRepository } = useContainer();
  const { getUseCase, setUseCase } = useMemo(
    () => ({
      getUseCase: makeGetProfileSettingsUseCase(profileSettingsRepository),
      setUseCase: makeSetProfileSettingsUseCase(profileSettingsRepository),
    }),
    [profileSettingsRepository],
  );
  const queryClient = useQueryClient();
  const { run, isPending: isSubmitting } = useCommand();
  const [error, setError] = useState<string | null>(null);

  const reportError = useCallback(
    (e: unknown) => {
      if (!isSessionExpired(e)) setError(errorMessage(e, t("errors.generic")));
    },
    [t],
  );

  // Load (and reset on) active-profile change. Without a profile there is
  // nothing to load — defaults apply.
  const query = useQuery({
    queryKey: settingsKey(profileId),
    enabled: profileId !== null,
    queryFn: async (): Promise<ProfileSettings> => {
      const id = profileId as string;
      const loaded = await getUseCase(id);
      const legacy = readLegacyPrefs();
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(LEGACY_MIGRATED_KEY, "1");
      }
      if (!legacy) return loaded;
      // Fold the legacy device prefs into this profile, once.
      const merged: ProfileSettings = { ...loaded, ...legacy };
      void setUseCase(id, merged).catch(() => {});
      return merged;
    },
  });

  useEffect(() => {
    if (query.error) reportError(query.error);
  }, [query.error, reportError]);

  const settings =
    profileId && query.data ? query.data : DEFAULT_PROFILE_SETTINGS;

  // Authoritative note-font reconcile: apply the active profile's font (and
  // refresh the pre-React paint cache). With no profile, restore the default.
  useEffect(() => {
    applyNoteFont(settings.noteFont);
  }, [settings.noteFont]);

  const update = useCallback(
    async (input: ProfileSettingsInput): Promise<boolean> => {
      if (!profileId) return false;
      const key = settingsKey(profileId);
      const previous = settings;
      const optimistic: ProfileSettings = {
        ...settings,
        ...input,
        co2: { ...settings.co2, ...(input.co2 ?? {}) },
      };
      queryClient.setQueryData(key, optimistic); // the note-font effect fires now
      setError(null);
      try {
        const canonical = await run(() => setUseCase(profileId, optimistic));
        queryClient.setQueryData(key, canonical);
        return true;
      } catch (e) {
        queryClient.setQueryData(key, previous); // roll back
        reportError(e);
        return false;
      }
    },
    [profileId, settings, queryClient, run, setUseCase, reportError],
  );

  const clearError = useCallback(() => setError(null), []);

  const value = useMemo<ProfileSettingsContextValue>(
    () => ({
      settings,
      isLoading: profileId !== null && query.isPending,
      isSubmitting,
      error,
      update,
      clearError,
    }),
    [
      settings,
      profileId,
      query.isPending,
      isSubmitting,
      error,
      update,
      clearError,
    ],
  );

  return (
    <ProfileSettingsContext.Provider value={value}>
      {children}
    </ProfileSettingsContext.Provider>
  );
}

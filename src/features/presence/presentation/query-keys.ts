/** TanStack Query keys of the presence feature. */
export const presenceKeys = {
  byProfile: (profileId: string | null) => ["presences", profileId] as const,
  trips: (presenceId: string) => ["presence-trips", presenceId] as const,
};

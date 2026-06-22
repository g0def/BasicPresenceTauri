import { useEffect, useState } from "react";

import { TauriCo2ReferentialRepository } from "@/features/methodology/data/repositories/tauri-co2-referential.repository";
import type { Co2Referential } from "@/features/methodology/domain/entities/co2-referential";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

const repo = new TauriCo2ReferentialRepository();

interface UseCo2ReferentialResult {
  referential: Co2Referential | null;
  isLoading: boolean;
  isError: boolean;
}

/** Load the full CO2 referential. Profile-independent data, but the vault must be
 * unlocked, so we (re)load once a profile/session is available. */
export function useCo2Referential(): UseCo2ReferentialResult {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const [referential, setReferential] = useState<Co2Referential | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isError, setIsError] = useState(false);

  useEffect(() => {
    if (!profileId) {
      setReferential(null);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setIsError(false);
    repo
      .get()
      .then((r) => {
        if (!cancelled) setReferential(r);
      })
      .catch(() => {
        if (!cancelled) setIsError(true);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [profileId]);

  return { referential, isLoading, isError };
}

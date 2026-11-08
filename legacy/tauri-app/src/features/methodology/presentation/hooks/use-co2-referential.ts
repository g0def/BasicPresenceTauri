import { useQuery } from "@tanstack/react-query";

import { useContainer } from "@/core/di";
import type { Co2Referential } from "@/features/methodology/domain/entities/co2-referential";
import { useProfile } from "@/entities/profile/model/use-profile";

interface UseCo2ReferentialResult {
  referential: Co2Referential | null;
  isLoading: boolean;
  isError: boolean;
}

/** Load the full CO2 referential. Profile-independent data, but the vault must
 * be unlocked, so it loads once a profile (hence a session) is available. */
export function useCo2Referential(): UseCo2ReferentialResult {
  const { activeProfile } = useProfile();
  const { co2ReferentialRepository } = useContainer();
  const query = useQuery({
    queryKey: ["co2-referential"],
    enabled: activeProfile !== null,
    queryFn: () => co2ReferentialRepository.get(),
  });
  return {
    referential: activeProfile ? (query.data ?? null) : null,
    isLoading: query.isFetching,
    isError: query.isError,
  };
}

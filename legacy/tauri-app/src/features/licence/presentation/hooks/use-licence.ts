import { useQuery } from "@tanstack/react-query";

import { useContainer } from "@/core/di";
import { useProfile } from "@/entities/profile/model/use-profile";

interface UseLicenceResult {
  /** Backend-rendered, sanitized HTML of the app licence. */
  html: string;
  isLoading: boolean;
  isError: boolean;
}

/** Load the app licence as sanitized HTML. The text is static and embedded in
 * the backend, but the command is session-gated, so it loads once a
 * profile/session is available. */
export function useLicence(): UseLicenceResult {
  const { activeProfile } = useProfile();
  const { licenceRepository } = useContainer();
  const query = useQuery({
    queryKey: ["licence"],
    enabled: activeProfile !== null,
    queryFn: () => licenceRepository.getHtml(),
  });
  return {
    html: activeProfile ? (query.data ?? "") : "",
    isLoading: query.isFetching,
    isError: query.isError,
  };
}

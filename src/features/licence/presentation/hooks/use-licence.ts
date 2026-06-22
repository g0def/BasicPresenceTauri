import { useEffect, useState } from "react";

import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

interface UseLicenceResult {
  /** Backend-rendered, sanitized HTML of the app licence. */
  html: string;
  isLoading: boolean;
  isError: boolean;
}

/** Load the app licence as sanitized HTML. The text is static and embedded in
 * the backend, but the command is session-gated, so we (re)load once a
 * profile/session is available. */
export function useLicence(): UseLicenceResult {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const [html, setHtml] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isError, setIsError] = useState(false);

  useEffect(() => {
    if (!profileId) {
      setHtml("");
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setIsError(false);
    invoke<string>(COMMANDS.getLicence)
      .then((value) => {
        if (!cancelled) setHtml(value);
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

  return { html, isLoading, isError };
}

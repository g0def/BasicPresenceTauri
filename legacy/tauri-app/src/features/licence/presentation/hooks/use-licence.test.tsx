import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import type { Profile } from "@/entities/profile/model/profile";
import {
  ProfileContext,
  type ProfileContextValue,
} from "@/entities/profile/model/profile-context";
import { useLicence } from "@/features/licence/presentation/hooks/use-licence";
import { withContainer } from "@/test/app-providers";

function profileValue(active: Profile | null): ProfileContextValue {
  return {
    profiles: active ? [active] : [],
    activeProfile: active,
    isLoading: false,
    isSubmitting: false,
    error: null,
    createProfile: () => Promise.resolve(true),
    updateProfile: () => Promise.resolve(true),
    deleteProfile: () => Promise.resolve(),
    setActiveProfile: () => Promise.resolve(),
    refresh: () => Promise.resolve(),
    clearError: () => {},
  };
}

const PROFILE: Profile = {
  id: "p1",
  firstName: "A",
  lastName: "B",
  enterprise: "C",
  poste: null,
  createdAt: 1,
  updatedAt: 1,
};

function wrapperFor(active: Profile | null, getHtml: () => Promise<string>) {
  const Providers = withContainer({ licenceRepository: { getHtml } });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <Providers>
        <ProfileContext.Provider value={profileValue(active)}>
          {children}
        </ProfileContext.Provider>
      </Providers>
    );
  };
}

describe("useLicence", () => {
  it("loads the sanitized licence once a session (profile) exists", async () => {
    const getHtml = vi.fn(() => Promise.resolve("<p>Licence</p>"));
    const { result } = renderHook(() => useLicence(), {
      wrapper: wrapperFor(PROFILE, getHtml),
    });
    await waitFor(() => expect(result.current.html).toBe("<p>Licence</p>"));
    expect(result.current.isError).toBe(false);
  });

  it("does not call the (session-gated) backend without a profile", () => {
    const getHtml = vi.fn(() => Promise.resolve("x"));
    const { result } = renderHook(() => useLicence(), {
      wrapper: wrapperFor(null, getHtml),
    });
    expect(result.current.html).toBe("");
    expect(getHtml).not.toHaveBeenCalled();
  });
});

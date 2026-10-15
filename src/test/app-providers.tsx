/* eslint-disable react-refresh/only-export-components -- test helpers, never hot-reloaded */
import type { ReactNode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";

import { createTauriContainer } from "@/app/container";
import { createQueryClient } from "@/app/query-client";
import { ContainerProvider } from "@/core/container-provider";
import type { Container } from "@/core/di";
import { AuthProvider } from "@/features/auth/presentation/providers/auth-provider";
import { SessionAwareUpdaterProvider } from "@/app/session-aware-updater";

/**
 * The provider tree of `main.tsx`, for whole-app tests: real (IPC-mocked)
 * repositories, and a fresh query client per test so no cache leaks across.
 */
export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <ContainerProvider value={createTauriContainer()}>
      <QueryClientProvider client={createQueryClient()}>
        <AuthProvider>
          <SessionAwareUpdaterProvider>{children}</SessionAwareUpdaterProvider>
        </AuthProvider>
      </QueryClientProvider>
    </ContainerProvider>
  );
}

/** Providers needed by a single feature provider under test. */
export function DataProviders({ children }: { children: ReactNode }) {
  return (
    <ContainerProvider value={createTauriContainer()}>
      <QueryClientProvider client={createQueryClient()}>
        {children}
      </QueryClientProvider>
    </ContainerProvider>
  );
}

/**
 * Wrapper injecting fakes for some ports (the rest stay the IPC-mocked Tauri
 * implementations): `renderHook(useX, { wrapper: withContainer({ xRepository }) })`.
 */
export function withContainer(overrides: Partial<Container>) {
  const container = { ...createTauriContainer(), ...overrides };
  const queryClient = createQueryClient();
  return function FakeContainerProviders({
    children,
  }: {
    children: ReactNode;
  }) {
    return (
      <ContainerProvider value={container}>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </ContainerProvider>
    );
  };
}

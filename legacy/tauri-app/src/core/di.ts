import { createContext, useContext } from "react";

/**
 * Dependency-injection container: the repositories (ports) the features use.
 *
 * Empty here on purpose: each feature declares the ports it needs in its own
 * `di.ts` through module augmentation, so dependencies keep pointing from the
 * features to `core`:
 *
 * ```ts
 * declare module "@/core/di" {
 *   interface Container {
 *     profileRepository: ProfileRepository;
 *   }
 * }
 * ```
 *
 * The concrete (Tauri) implementations are wired once, in the composition root
 * (`src/app/container.ts`); tests provide fakes through {@link ContainerProvider}.
 */
// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- augmented per feature
export interface Container {}

export const ContainerContext = createContext<Container | null>(null);

export function useContainer(): Container {
  const container = useContext(ContainerContext);
  if (!container) {
    throw new Error("useContainer must be used within a ContainerProvider");
  }
  return container;
}

import type { ReactNode } from "react";

import { type Container, ContainerContext } from "@/core/di";

/** Provides the DI container (see core/di) to the tree. */
export function ContainerProvider({
  value,
  children,
}: {
  value: Container;
  children: ReactNode;
}) {
  return (
    <ContainerContext.Provider value={value}>
      {children}
    </ContainerContext.Provider>
  );
}

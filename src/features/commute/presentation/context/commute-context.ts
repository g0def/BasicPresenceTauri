import { createContext } from "react";

import type {
  Commute,
  CreateCommuteInput,
  EmissionFactor,
  UpdateCommuteInput,
} from "@/features/commute/domain/entities/commute";

export interface CommuteContextValue {
  /** Saved commutes of the active profile. */
  commutes: Commute[];
  /** The emission-factor referential (for the mode pickers). */
  emissionFactors: EmissionFactor[];
  isLoading: boolean;
  isSubmitting: boolean;
  error: string | null;
  /** Create a commute for the active profile. Returns `true` on success. */
  createCommute: (input: CreateCommuteInput) => Promise<boolean>;
  /** Update a commute. Returns `true` on success. */
  updateCommute: (input: UpdateCommuteInput) => Promise<boolean>;
  /** Delete a commute by id. Returns `true` on success. */
  deleteCommute: (id: string) => Promise<boolean>;
  clearError: () => void;
}

export const CommuteContext = createContext<CommuteContextValue | null>(null);

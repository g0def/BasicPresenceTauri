import { Building2, House, Palmtree, PartyPopper } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import type { PresenceType } from "@/features/presence/domain/entities/presence";

type PresenceLabelKey =
  | "presence.types.office"
  | "presence.types.remote"
  | "presence.types.vacation"
  | "presence.types.holiday";

interface PresenceStyle {
  /** Tailwind classes painting a filled day cell (bg + readable foreground). */
  cell: string;
  /** Text color token, for the icon in non-filled contexts (legend, dialog). */
  text: string;
  /** A discreet icon evoking the type, shown on filled cells and in the legend. */
  Icon: LucideIcon;
  /** i18n key for the human label of this type. */
  labelKey: PresenceLabelKey;
}

/**
 * One color + icon per presence type, reusing the existing semantic tokens from
 * index.css (so light/dark variants are handled by the CSS variables). Icons are
 * from lucide-react (already a dependency; ISC-licensed, shadcn's default set).
 * The class strings are static literals so Tailwind's scanner picks them up.
 */
export const PRESENCE_STYLES: Record<PresenceType, PresenceStyle> = {
  office: {
    cell: "bg-primary text-primary-foreground",
    text: "text-primary",
    Icon: Building2,
    labelKey: "presence.types.office",
  },
  remote: {
    cell: "bg-success text-success-foreground",
    text: "text-success",
    Icon: House,
    labelKey: "presence.types.remote",
  },
  vacation: {
    cell: "bg-warning text-white",
    text: "text-warning",
    Icon: Palmtree,
    labelKey: "presence.types.vacation",
  },
  holiday: {
    cell: "bg-destructive text-white",
    text: "text-destructive",
    Icon: PartyPopper,
    labelKey: "presence.types.holiday",
  },
};

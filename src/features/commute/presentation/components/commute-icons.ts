import { Bike, Bus, Car, HelpCircle, Plane, TrainFront } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import type { CommuteCategory } from "@/features/commute/domain/entities/commute";

/** One icon per transport category (lucide-react, already a dependency). */
export const CATEGORY_ICONS: Record<CommuteCategory, LucideIcon> = {
  car: Car,
  active: Bike,
  public_transport: Bus,
  rail: TrainFront,
  air: Plane,
  other: HelpCircle,
};

type CategoryLabelKey =
  | "commute.categories.car"
  | "commute.categories.active"
  | "commute.categories.public_transport"
  | "commute.categories.rail"
  | "commute.categories.air"
  | "commute.categories.other";

/** i18n key for each category's group header. */
export const CATEGORY_LABEL_KEYS: Record<CommuteCategory, CategoryLabelKey> = {
  car: "commute.categories.car",
  active: "commute.categories.active",
  public_transport: "commute.categories.public_transport",
  rail: "commute.categories.rail",
  air: "commute.categories.air",
  other: "commute.categories.other",
};

/** Display order of the category groups in the pickers. */
export const CATEGORY_ORDER: CommuteCategory[] = [
  "car",
  "active",
  "public_transport",
  "rail",
  "air",
  "other",
];

/** Occupants only matter for per-vehicle (car) modes. */
export function isCarCategory(category: CommuteCategory | undefined): boolean {
  return category === "car";
}

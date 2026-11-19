import { useTranslation } from "react-i18next";

import type { PresenceType } from "@/entities/presence/model/presence";
import { PRESENCE_TYPES } from "@/entities/presence/model/presence";
import { PRESENCE_STYLES } from "@/entities/presence/ui/presence-colors";
import { cn } from "@/lib/utils";

interface PresenceTypeFilterProps<T extends PresenceType> {
  enabled: ReadonlySet<T>;
  onToggle: (type: T) => void;
  /** Types offered, in display order (default: all four). */
  types?: readonly T[];
  className?: string;
}

/** Include/exclude checkboxes, one per presence type, reusing the calendar
 * legend's icon + the app's raw-checkbox idiom (Stats breakdown, Odoo export). */
export function PresenceTypeFilter<T extends PresenceType = PresenceType>({
  enabled,
  onToggle,
  types = PRESENCE_TYPES as readonly PresenceType[] as readonly T[],
  className,
}: PresenceTypeFilterProps<T>) {
  const { t } = useTranslation();
  return (
    <div
      className={cn("flex flex-wrap justify-center gap-x-5 gap-y-2", className)}
    >
      {types.map((type) => {
        const style = PRESENCE_STYLES[type];
        const Icon = style.Icon;
        return (
          <label
            key={type}
            className="flex items-center gap-2 text-sm text-muted-foreground"
          >
            <input
              type="checkbox"
              className="size-4 rounded border-input accent-primary"
              checked={enabled.has(type)}
              onChange={() => onToggle(type)}
            />
            <Icon className={cn("size-4", style.text)} aria-hidden />
            {t(style.labelKey)}
          </label>
        );
      })}
    </div>
  );
}

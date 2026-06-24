import { useTranslation } from "react-i18next";

import type { PresenceType } from "@/features/presence/domain/entities/presence";
import { PRESENCE_TYPES } from "@/features/presence/domain/entities/presence";
import { PRESENCE_STYLES } from "@/features/presence/presentation/components/presence-colors";
import { cn } from "@/lib/utils";

interface PresenceTypeFilterProps {
  enabled: ReadonlySet<PresenceType>;
  onToggle: (type: PresenceType) => void;
}

/** Include/exclude checkboxes for the breakdown pie — one per presence type,
 * reusing the calendar legend's icon + the app's raw-checkbox idiom. */
export function PresenceTypeFilter({
  enabled,
  onToggle,
}: PresenceTypeFilterProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap justify-center gap-x-5 gap-y-2">
      {PRESENCE_TYPES.map((type) => {
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

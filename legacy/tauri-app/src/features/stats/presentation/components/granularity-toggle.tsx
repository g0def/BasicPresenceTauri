import { useTranslation } from "react-i18next";

import type { Granularity } from "@/features/stats/domain/aggregate";
import { cn } from "@/lib/utils";

const OPTIONS: Granularity[] = ["month", "week"];

interface GranularityToggleProps {
  value: Granularity;
  onChange: (granularity: Granularity) => void;
}

/** Month/week segmented control, styled like the settings-page mode pills. */
export function GranularityToggle({ value, onChange }: GranularityToggleProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap gap-2">
      {OPTIONS.map((option) => (
        <button
          key={option}
          type="button"
          onClick={() => onChange(option)}
          aria-pressed={value === option}
          className={cn(
            "rounded-md border px-4 py-2 text-sm transition-colors",
            value === option
              ? "border-primary bg-primary text-primary-foreground"
              : "bg-card hover:bg-accent",
          )}
        >
          {t(`stats.granularity.${option}`)}
        </button>
      ))}
    </div>
  );
}

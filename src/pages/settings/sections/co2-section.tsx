import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useProfileSettings } from "@/features/profile-settings/presentation/hooks/use-profile-settings";

const GRID_COUNTRIES = ["FR", "BE", "DE", "EU"] as const;

/** Per-profile CO2 configuration (grid country, car occupancy, aviation RF). */
export function Co2Section() {
  const { t } = useTranslation();
  const { settings, update } = useProfileSettings();

  // Car-occupancy is a free number field: keep a local draft, commit on blur.
  const [occupancy, setOccupancy] = useState(
    String(settings.co2.defaultCarOccupancy),
  );
  useEffect(() => {
    setOccupancy(String(settings.co2.defaultCarOccupancy));
  }, [settings.co2.defaultCarOccupancy]);
  const commitOccupancy = () => {
    const n = Math.round(Number(occupancy));
    if (
      Number.isFinite(n) &&
      n >= 1 &&
      n !== settings.co2.defaultCarOccupancy
    ) {
      void update({ co2: { defaultCarOccupancy: n } });
    } else {
      setOccupancy(String(settings.co2.defaultCarOccupancy));
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-base font-medium">{t("settings.co2Section")}</h3>

      <div className="grid gap-1.5">
        <span className="text-xs text-muted-foreground">
          {t("settings.co2.gridCountry")}
        </span>
        <div className="flex flex-wrap gap-2">
          {GRID_COUNTRIES.map((country) => (
            <button
              key={country}
              type="button"
              onClick={() => void update({ co2: { gridCountry: country } })}
              className={`rounded-md border px-4 py-2 text-sm transition-colors ${
                settings.co2.gridCountry === country
                  ? "border-primary bg-primary text-primary-foreground"
                  : "bg-card hover:bg-accent"
              }`}
            >
              {t(`settings.co2.countries.${country}`)}
            </button>
          ))}
        </div>
      </div>

      <div className="grid gap-1.5">
        <Label
          htmlFor="car-occupancy"
          className="text-xs text-muted-foreground"
        >
          {t("settings.co2.carOccupancy")}
        </Label>
        <Input
          id="car-occupancy"
          type="number"
          min={1}
          className="w-24 tabular-nums"
          value={occupancy}
          onChange={(e) => setOccupancy(e.target.value)}
          onBlur={commitOccupancy}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.currentTarget.blur();
          }}
        />
      </div>

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          className="size-4"
          checked={settings.co2.includeRadiativeForcing}
          onChange={(e) =>
            void update({
              co2: { includeRadiativeForcing: e.target.checked },
            })
          }
        />
        {t("settings.co2.radiativeForcing")}
      </label>
    </div>
  );
}

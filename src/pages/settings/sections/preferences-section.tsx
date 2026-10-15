import { useTranslation } from "react-i18next";

import { useProfileSettings } from "@/features/profile-settings/presentation/hooks/use-profile-settings";
import { StartTimePicker } from "@/features/work-hours/presentation/components/start-time-picker";
import type { CellDisplayMode } from "@/shared/cell-display/cell-display-context";
import { NOTE_FONTS } from "@/shared/note-font/use-note-font";

const CELL_DISPLAY_MODES: CellDisplayMode[] = ["co2", "hours"];

/** Per-profile display preferences: default log hour, note font, cell mode. */
export function PreferencesSection() {
  const { t } = useTranslation();
  const { settings, update } = useProfileSettings();

  return (
    <>
      {/* Default log hour */}
      <div className="flex flex-col gap-3">
        <h3 className="text-base font-medium">
          {t("settings.logHourSection")}
        </h3>
        <p className="text-sm text-muted-foreground">
          {t("settings.defaultStartTimeHint")}
        </p>
        <StartTimePicker
          id="default-start-time"
          value={settings.startMinutes}
          onCommit={(startMinutes) => void update({ startMinutes })}
          label={t("settings.defaultStartTime")}
        />
      </div>

      {/* Note font */}
      <div className="flex flex-col gap-3">
        <h3 className="text-base font-medium">
          {t("settings.noteFontSection")}
        </h3>
        <div className="flex flex-wrap gap-2">
          {NOTE_FONTS.map((font) => (
            <button
              key={font}
              type="button"
              onClick={() => void update({ noteFont: font })}
              className={`flex items-center gap-3 rounded-md border px-4 py-2 text-sm transition-colors ${
                settings.noteFont === font
                  ? "border-primary bg-primary text-primary-foreground"
                  : "bg-card hover:bg-accent"
              }`}
            >
              <span>{t(`noteFont.${font}`)}</span>
              {/* Preview resolves the same --note-font the notes use
                  (index.css), so the sample never drifts from the real font. */}
              <span
                data-note-font={font}
                className="opacity-60"
                style={{ fontFamily: "var(--note-font)" }}
              >
                Aa
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Calendar cell display */}
      <div className="flex flex-col gap-3">
        <h3 className="text-base font-medium">
          {t("settings.cellDisplaySection")}
        </h3>
        <div className="flex flex-wrap gap-2">
          {CELL_DISPLAY_MODES.map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => void update({ cellDisplayMode: mode })}
              className={`rounded-md border px-4 py-2 text-sm transition-colors ${
                settings.cellDisplayMode === mode
                  ? "border-primary bg-primary text-primary-foreground"
                  : "bg-card hover:bg-accent"
              }`}
            >
              {t(`cellDisplay.${mode}`)}
            </button>
          ))}
        </div>
      </div>
    </>
  );
}

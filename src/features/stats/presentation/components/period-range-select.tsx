import { useTranslation } from "react-i18next";

export interface PeriodOption {
  key: number;
  label: string;
}

interface PeriodRangeSelectProps {
  periods: PeriodOption[];
  start: number;
  end: number;
  onChange: (start: number, end: number) => void;
}

const SELECT_CLASS =
  "rounded-md border border-input bg-card px-2 py-1 text-sm text-foreground";

/** "De [période] à [période]" range pickers (period keys as values). Keeps
 * start ≤ end by nudging the other bound when a selection would cross it. Reused
 * for the bar charts' window and the breakdown pie's range. */
export function PeriodRangeSelect({
  periods,
  start,
  end,
  onChange,
}: PeriodRangeSelectProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-wrap items-center justify-center gap-2">
      <label className="flex items-center gap-1.5 text-sm text-muted-foreground">
        {t("stats.from")}
        <select
          className={SELECT_CLASS}
          value={String(start)}
          onChange={(e) => {
            const next = Number(e.target.value);
            onChange(next, Math.max(next, end));
          }}
        >
          {periods.map((p) => (
            <option key={p.key} value={String(p.key)}>
              {p.label}
            </option>
          ))}
        </select>
      </label>
      <label className="flex items-center gap-1.5 text-sm text-muted-foreground">
        {t("stats.to")}
        <select
          className={SELECT_CLASS}
          value={String(end)}
          onChange={(e) => {
            const next = Number(e.target.value);
            onChange(Math.min(start, next), next);
          }}
        >
          {periods.map((p) => (
            <option key={p.key} value={String(p.key)}>
              {p.label}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}

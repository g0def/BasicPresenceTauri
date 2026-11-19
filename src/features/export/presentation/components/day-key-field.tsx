import { useTranslation } from "react-i18next";

import { Input } from "@/components/ui/input";
import {
  formatDayKey,
  parseDayKey,
} from "@/features/export/domain/entities/attendance-export";

interface DayKeyFieldProps {
  id: string;
  /** Epoch ms at UTC midnight. */
  value: number;
  onChange: (day: number) => void;
}

/**
 * A day typed as `dd/mm/yyyy`. Used instead of `<input type="date">`, whose
 * display format follows the WebView's OS locale (mm/dd/yyyy on an English
 * system) and cannot be forced. Committed on blur or Enter; an invalid entry
 * reverts to the current value. Uncontrolled and keyed on `value`, so an
 * outside change (e.g. "This month") re-renders it with the new date.
 */
export function DayKeyField({ id, value, onChange }: DayKeyFieldProps) {
  const { t } = useTranslation();
  const commit = (input: HTMLInputElement) => {
    const day = parseDayKey(input.value);
    if (day === null) input.value = formatDayKey(value);
    else if (day !== value) onChange(day);
    else input.value = formatDayKey(value);
  };
  return (
    <Input
      key={value}
      id={id}
      type="text"
      inputMode="numeric"
      placeholder={t("export.odoo.datePlaceholder")}
      maxLength={10}
      className="w-32 tabular-nums"
      defaultValue={formatDayKey(value)}
      onBlur={(e) => commit(e.currentTarget)}
      onKeyDown={(e) => {
        if (e.key === "Enter") commit(e.currentTarget);
      }}
    />
  );
}

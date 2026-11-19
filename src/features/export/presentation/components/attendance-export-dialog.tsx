import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PresenceTypeFilter } from "@/entities/presence/ui/presence-type-filter";
import {
  ATTENDANCE_KINDS,
  type AttendanceDay,
  type AttendanceExportSummary,
  type AttendanceKind,
  DEFAULT_FALLBACK,
  dayKeyToIso,
  filterAttendanceDays,
  monthRange,
  usesFallback,
} from "@/features/export/domain/entities/attendance-export";
import { DayKeyField } from "@/features/export/presentation/components/day-key-field";
import { useAttendanceExport } from "@/features/export/presentation/hooks/use-attendance-export";
import { TimePicker } from "@/shared/components/time-picker";
import { formatClockTime } from "@/shared/format/duration-format";
import { useResetOnOpen } from "@/shared/hooks/use-reset-on-open";

interface AttendanceExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  profileId: string;
  /** Pre-fills the `employee_id` column (the profile's "First Last"). */
  defaultEmployeeName: string;
}

type Stage = "idle" | "result";

const ALL_KINDS: ReadonlySet<AttendanceKind> = new Set(ATTENDANCE_KINDS);

/**
 * Export worked days as an Odoo attendance CSV. The period and day types
 * pre-select the matching days; each day can then be unticked. The backend
 * re-reads the hours (falling back to the hours chosen here for days without
 * a usable schedule) and writes the file after a native save dialog.
 */
export function AttendanceExportDialog({
  open,
  onOpenChange,
  profileId,
  defaultEmployeeName,
}: AttendanceExportDialogProps) {
  const { t, i18n } = useTranslation();
  const { days, isLoading, exportAttendance, isExporting, error, clearError } =
    useAttendanceExport(profileId, open);

  const [stage, setStage] = useState<Stage>("idle");
  const [summary, setSummary] = useState<AttendanceExportSummary | null>(null);
  const [employeeName, setEmployeeName] = useState(defaultEmployeeName);
  const [range, setRange] = useState(() => monthRange(new Date()));
  const [kinds, setKinds] = useState<ReadonlySet<AttendanceKind>>(ALL_KINDS);
  const [startMinutes, setStartMinutes] = useState<number>(
    DEFAULT_FALLBACK.start,
  );
  const [endMinutes, setEndMinutes] = useState<number>(DEFAULT_FALLBACK.end);
  const [applyToAll, setApplyToAll] = useState(false);
  // Days of the filtered list the user unticked. Cleared whenever the
  // filters change, so the selection restarts from the filtered result.
  const [unchecked, setUnchecked] = useState<ReadonlySet<number>>(new Set());

  useResetOnOpen(open, null, () => {
    setStage("idle");
    setSummary(null);
    setEmployeeName(defaultEmployeeName);
    setRange(monthRange(new Date()));
    setKinds(ALL_KINDS);
    setStartMinutes(DEFAULT_FALLBACK.start);
    setEndMinutes(DEFAULT_FALLBACK.end);
    setApplyToAll(false);
    setUnchecked(new Set());
    clearError();
  });

  const filtered = useMemo(
    () => filterAttendanceDays(days, { ...range, kinds }),
    [days, range, kinds],
  );
  const selected = filtered.filter((d) => !unchecked.has(d.day));

  const hoursValid = startMinutes < endMinutes;
  const canExport =
    employeeName.trim() !== "" &&
    hoursValid &&
    selected.length > 0 &&
    !isExporting;

  const dateLabel = useMemo(
    () =>
      new Intl.DateTimeFormat(i18n.resolvedLanguage ?? "fr", {
        weekday: "short",
        day: "numeric",
        month: "short",
        year: "numeric",
        timeZone: "UTC", // day keys are UTC midnights
      }),
    [i18n.resolvedLanguage],
  );

  const changeRange = (next: { from: number; to: number }) => {
    setRange(next);
    setUnchecked(new Set());
  };
  const changeDate = (bound: "from" | "to", key: number) => {
    const next = { ...range, [bound]: key };
    // Keep from <= to by dragging the other bound along.
    if (next.from > next.to) {
      if (bound === "from") next.to = key;
      else next.from = key;
    }
    changeRange(next);
  };
  const toggleKind = (kind: AttendanceKind) => {
    setKinds((prev) => {
      const next = new Set(prev);
      if (next.has(kind)) next.delete(kind);
      else next.add(kind);
      return next;
    });
    setUnchecked(new Set());
  };
  const toggleDay = (day: number) =>
    setUnchecked((prev) => {
      const next = new Set(prev);
      if (next.has(day)) next.delete(day);
      else next.add(day);
      return next;
    });

  const runExport = async () => {
    if (!canExport) return;
    const fileName = `${t("export.odoo.fileBaseName")}-${dayKeyToIso(range.from)}-${dayKeyToIso(range.to)}.csv`;
    const result = await exportAttendance(
      {
        employeeName: employeeName.trim(),
        days: selected.map((d) => d.day),
        fallbackStartMinutes: startMinutes,
        fallbackEndMinutes: endMinutes,
        applyToAll,
      },
      fileName,
    );
    if (result) {
      setSummary(result);
      setStage("result");
    }
  };

  const fallbackLabel = hoursValid
    ? `${formatClockTime(startMinutes)}–${formatClockTime(endMinutes)}`
    : "—";
  const hoursOf = (d: AttendanceDay) =>
    usesFallback(d, applyToAll) ||
    d.startMinutes === null ||
    d.endMinutes === null
      ? fallbackLabel
      : `${formatClockTime(d.startMinutes)}–${formatClockTime(d.endMinutes)}`;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("export.odoo.title")}</DialogTitle>
          {stage === "idle" && (
            <>
              <DialogDescription>
                {t("export.odoo.description")}
              </DialogDescription>
              <p className="text-xs text-warning">
                {t("export.unencryptedWarning")}
              </p>
            </>
          )}
        </DialogHeader>

        {stage === "idle" && (
          <div className="grid gap-5 text-sm">
            <div className="grid gap-2">
              <Label htmlFor="odoo-employee">{t("export.odoo.employee")}</Label>
              <Input
                id="odoo-employee"
                value={employeeName}
                maxLength={120}
                onChange={(e) => setEmployeeName(e.target.value)}
                aria-invalid={employeeName.trim() === ""}
              />
              <p className="text-xs text-muted-foreground">
                {t("export.odoo.employeeHint")}
              </p>
            </div>

            <fieldset className="grid gap-2">
              <legend className="mb-2 font-medium">
                {t("export.odoo.period")}
              </legend>
              <div className="flex flex-wrap items-end gap-3">
                <div className="grid gap-1">
                  <Label htmlFor="odoo-from">{t("export.odoo.from")}</Label>
                  <DayKeyField
                    id="odoo-from"
                    value={range.from}
                    onChange={(day) => changeDate("from", day)}
                  />
                </div>
                <div className="grid gap-1">
                  <Label htmlFor="odoo-to">{t("export.odoo.to")}</Label>
                  <DayKeyField
                    id="odoo-to"
                    value={range.to}
                    onChange={(day) => changeDate("to", day)}
                  />
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => changeRange(monthRange(new Date()))}
                >
                  {t("export.odoo.thisMonth")}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => changeRange(monthRange(new Date(), -1))}
                >
                  {t("export.odoo.lastMonth")}
                </Button>
              </div>
            </fieldset>

            <fieldset className="grid gap-2">
              <legend className="mb-2 font-medium">
                {t("export.odoo.types")}
              </legend>
              <PresenceTypeFilter
                types={ATTENDANCE_KINDS}
                enabled={kinds}
                onToggle={toggleKind}
                className="justify-start"
              />
            </fieldset>

            <fieldset className="grid gap-2">
              <legend className="mb-2 font-medium">
                {t("export.odoo.fallback")}
              </legend>
              <div className="flex flex-wrap items-end gap-3">
                <TimePicker
                  id="odoo-start"
                  label={t("export.odoo.fallbackStart")}
                  value={startMinutes}
                  onCommit={setStartMinutes}
                  invalid={!hoursValid}
                />
                <TimePicker
                  id="odoo-end"
                  label={t("export.odoo.fallbackEnd")}
                  value={endMinutes}
                  onCommit={setEndMinutes}
                  invalid={!hoursValid}
                />
              </div>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  className="size-4 rounded border-input accent-primary"
                  checked={applyToAll}
                  onChange={(e) => setApplyToAll(e.target.checked)}
                />
                {t("export.odoo.applyToAll")}
              </label>
              <p className="text-xs text-muted-foreground">
                {hoursValid ? (
                  t(
                    applyToAll
                      ? "export.odoo.applyToAllHint"
                      : "export.odoo.fallbackHint",
                  )
                ) : (
                  <span className="text-destructive">
                    {t("export.odoo.invalidHours")}
                  </span>
                )}
              </p>
            </fieldset>

            <fieldset className="grid gap-2">
              <legend className="mb-2 font-medium">
                {t("export.odoo.days")}
              </legend>
              {isLoading ? (
                <p className="text-muted-foreground">{t("common.loading")}</p>
              ) : filtered.length === 0 ? (
                <p className="text-muted-foreground">
                  {t("export.odoo.noDays")}
                </p>
              ) : (
                <>
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="mr-auto text-muted-foreground">
                      {t("export.odoo.selectedCount", {
                        count: selected.length,
                        total: filtered.length,
                      })}
                    </span>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setUnchecked(new Set())}
                    >
                      {t("export.odoo.selectAll")}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        setUnchecked(new Set(filtered.map((d) => d.day)))
                      }
                    >
                      {t("export.odoo.selectNone")}
                    </Button>
                  </div>
                  <ul className="max-h-64 overflow-y-auto rounded-md border divide-y">
                    {filtered.map((d) => (
                      <li key={d.day}>
                        <label className="flex items-center gap-3 px-3 py-2">
                          <input
                            type="checkbox"
                            className="size-4 rounded border-input accent-primary"
                            checked={!unchecked.has(d.day)}
                            onChange={() => toggleDay(d.day)}
                          />
                          <span className="w-36 capitalize">
                            {dateLabel.format(d.day)}
                          </span>
                          <span className="w-24 text-muted-foreground">
                            {t(`presence.types.${d.kind}`)}
                          </span>
                          <span className="tabular-nums">{hoursOf(d)}</span>
                          {usesFallback(d, applyToAll) && (
                            <span className="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                              {t("export.odoo.defaultBadge")}
                            </span>
                          )}
                        </label>
                      </li>
                    ))}
                  </ul>
                </>
              )}
            </fieldset>
          </div>
        )}

        {stage === "result" && summary && (
          <div className="grid gap-2 text-sm">
            <p>
              {t("export.odoo.result", {
                rows: summary.rows,
                fallbackRows: summary.fallbackRows,
              })}
            </p>
            <p className="break-all text-muted-foreground">{summary.path}</p>
            <p className="text-xs text-muted-foreground">
              {t("export.odoo.timezoneHint")}
            </p>
          </div>
        )}

        {error && <p className="text-sm text-destructive">{error}</p>}

        <DialogFooter>
          {stage === "idle" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("export.cancel")}
              </Button>
              <Button disabled={!canExport} onClick={() => void runExport()}>
                {isExporting ? t("export.exporting") : t("export.exportButton")}
              </Button>
            </>
          )}
          {stage === "result" && (
            <Button onClick={() => onOpenChange(false)}>
              {t("export.done")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

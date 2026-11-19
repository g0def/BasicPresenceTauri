import { useCallback, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import type {
  AttendanceDay,
  AttendanceExportRequest,
  AttendanceExportSummary,
} from "@/features/export/domain/entities/attendance-export";

const NO_DAYS: AttendanceDay[] = [];

export const attendanceKeys = {
  days: (profileId: string) => ["attendance-days", profileId] as const,
};

/**
 * Drives the Odoo attendance export: loads the profile's worked days (only
 * while `enabled`, i.e. the dialog is open — always refetched so schedules
 * edited meanwhile show up), then runs the export. `exportAttendance` resolves
 * to `null` when the user cancelled the save dialog (no error in that case).
 */
export function useAttendanceExport(profileId: string, enabled: boolean) {
  const { t } = useTranslation();
  const { exportRepository } = useContainer();
  const { run, isPending: isExporting } = useCommand();
  const [error, setError] = useState<string | null>(null);

  const query = useQuery({
    queryKey: attendanceKeys.days(profileId),
    enabled,
    staleTime: 0,
    gcTime: 0,
    queryFn: () => exportRepository.listAttendanceDays(profileId),
  });

  const exportAttendance = useCallback(
    async (
      request: AttendanceExportRequest,
      defaultFileName: string,
    ): Promise<AttendanceExportSummary | null> => {
      setError(null);
      try {
        return await run(() =>
          exportRepository.exportAttendance(
            profileId,
            request,
            defaultFileName,
          ),
        );
      } catch (e) {
        if (!isSessionExpired(e))
          setError(errorMessage(e, t("errors.generic")));
        return null;
      }
    },
    [exportRepository, profileId, run, t],
  );

  const clearError = useCallback(() => setError(null), []);

  const loadError =
    query.error && !isSessionExpired(query.error)
      ? errorMessage(query.error, t("errors.generic"))
      : null;

  return {
    days: query.data ?? NO_DAYS,
    isLoading: query.isLoading,
    exportAttendance,
    isExporting,
    error: error ?? loadError,
    clearError,
  };
}

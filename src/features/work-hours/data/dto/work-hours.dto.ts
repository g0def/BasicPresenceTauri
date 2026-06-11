/** Wire shape of `TaskPresetDto` (serde camelCase). */
export interface TaskPresetDto {
  id: string;
  profileId: string;
  title: string;
  description: string | null;
  defaultMinutes: number;
  color: string;
  createdAt: number;
  updatedAt: number;
}

/** Wire shape of `WorkEntryDto` (serde camelCase). */
export interface WorkEntryDto {
  id: string;
  title: string;
  description: string | null;
  minutes: number;
  color: string;
  position: number;
}

/** Wire shape of `WorkDayScheduleDto` (serde camelCase). */
export interface WorkDayScheduleDto {
  startMinutes: number;
  endMinutes: number;
}

/** Wire shape of `WorkDayDto` (serde camelCase). */
export interface WorkDayDto {
  entries: WorkEntryDto[];
  schedule: WorkDayScheduleDto | null;
}

// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type { DayNoteDto } from "@/features/work-hours/data/dto/day-note.dto";
import type {
  TaskPresetDto,
  WorkDayDto,
  WorkDayScheduleDto,
  WorkEntryDto,
} from "@/features/work-hours/data/dto/work-hours.dto";

interface TaskPresetFieldsArgs {
  title: string;
  description: string | null;
  defaultMinutes: number;
  color: string;
}

interface WorkEntryArgs {
  title: string;
  description: string | null;
  minutes: number;
  color: string;
}

declare module "@/core/ipc" {
  interface CommandRegistry {
    list_task_presets: { args: { profileId: string }; result: TaskPresetDto[] };
    create_task_preset: {
      args: TaskPresetFieldsArgs & { profileId: string };
      result: TaskPresetDto;
    };
    update_task_preset: {
      args: TaskPresetFieldsArgs & { id: string };
      result: TaskPresetDto;
    };
    delete_task_preset: { args: { id: string }; result: void };
    get_work_entries: { args: { presenceId: string }; result: WorkEntryDto[] };
    set_work_entries: {
      args: { presenceId: string; entries: WorkEntryArgs[] };
      result: WorkDayDto;
    };
    get_work_schedule: {
      args: { presenceId: string };
      result: WorkDayScheduleDto | null;
    };
    set_work_schedule: {
      args: { presenceId: string; startMinutes: number };
      result: WorkDayScheduleDto;
    };
    get_day_note: { args: { presenceId: string }; result: DayNoteDto };
    set_day_note: {
      args: { presenceId: string; markdown: string };
      result: DayNoteDto;
    };
  }
}

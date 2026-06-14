import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { DayNoteDto } from "@/features/work-hours/data/dto/day-note.dto";
import type { DayNote } from "@/features/work-hours/domain/entities/day-note";
import type { DayNoteRepository } from "@/features/work-hours/domain/repositories/day-note-repository";

/** DayNoteRepository implementation backed by Tauri IPC commands. The DTO and
 * the entity share the same `{ markdown, html }` shape, so no mapping is needed. */
export class TauriDayNoteRepository implements DayNoteRepository {
  async get(presenceId: string): Promise<DayNote> {
    return invoke<DayNoteDto>(COMMANDS.getDayNote, { presenceId });
  }

  async set(presenceId: string, markdown: string): Promise<DayNote> {
    return invoke<DayNoteDto>(COMMANDS.setDayNote, { presenceId, markdown });
  }
}

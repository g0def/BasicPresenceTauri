import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { Co2ReferentialDto } from "@/features/methodology/data/dto/co2-referential.dto";
import { toCo2Referential } from "@/features/methodology/data/mappers/co2-referential.mapper";
import type { Co2Referential } from "@/features/methodology/domain/entities/co2-referential";

/** Reads the full CO2 referential (for the methodology page) over Tauri IPC. */
export class TauriCo2ReferentialRepository {
  async get(): Promise<Co2Referential> {
    const dto = await invoke<Co2ReferentialDto>(COMMANDS.listCo2Referential);
    return toCo2Referential(dto);
  }
}

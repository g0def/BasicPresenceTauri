import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import { toCo2Referential } from "@/features/methodology/data/mappers/co2-referential.mapper";
import type { Co2Referential } from "@/features/methodology/domain/entities/co2-referential";
import type { Co2ReferentialRepository } from "@/features/methodology/domain/repositories/co2-referential-repository";

/** Reads the full CO2 referential (for the methodology page) over Tauri IPC. */
export class TauriCo2ReferentialRepository implements Co2ReferentialRepository {
  async get(): Promise<Co2Referential> {
    const dto = await invoke(COMMANDS.listCo2Referential);
    return toCo2Referential(dto);
  }
}

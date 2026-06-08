import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { EmissionFactorDto } from "@/features/commute/data/dto/commute.dto";
import { toEmissionFactor } from "@/features/commute/data/mappers/commute.mapper";
import type { EmissionFactor } from "@/features/commute/domain/entities/commute";
import type { EmissionFactorRepository } from "@/features/commute/domain/repositories/emission-factor-repository";

/** EmissionFactorRepository implementation backed by Tauri IPC commands. */
export class TauriEmissionFactorRepository implements EmissionFactorRepository {
  async list(): Promise<EmissionFactor[]> {
    const dtos = await invoke<EmissionFactorDto[]>(
      COMMANDS.listEmissionFactors,
    );
    return dtos.map(toEmissionFactor);
  }
}

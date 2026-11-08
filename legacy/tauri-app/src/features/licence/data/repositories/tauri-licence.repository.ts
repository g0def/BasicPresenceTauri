import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { LicenceRepository } from "@/features/licence/domain/repositories/licence-repository";

/** Reads the app licence (static, embedded in the backend) over Tauri IPC. */
export class TauriLicenceRepository implements LicenceRepository {
  async getHtml(): Promise<string> {
    return invoke(COMMANDS.getLicence);
  }
}

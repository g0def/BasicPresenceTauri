import type { Container } from "@/core/di";
import { TauriAuthRepository } from "@/features/auth/data/repositories/tauri-auth.repository";
import { TauriCommuteRepository } from "@/features/commute/data/repositories/tauri-commute.repository";
import { TauriEmissionFactorRepository } from "@/features/commute/data/repositories/tauri-emission-factor.repository";
import { TauriExportRepository } from "@/features/export/data/repositories/tauri-export.repository";
import { TauriLicenceRepository } from "@/features/licence/data/repositories/tauri-licence.repository";
import { TauriCo2ReferentialRepository } from "@/features/methodology/data/repositories/tauri-co2-referential.repository";
import { TauriPresenceRepository } from "@/features/presence/data/repositories/tauri-presence.repository";
import { TauriProfileSettingsRepository } from "@/features/profile-settings/data/repositories/tauri-profile-settings.repository";
import { TauriTransferRepository } from "@/features/profile-transfer/data/repositories/tauri-transfer.repository";
import { TauriProfileRepository } from "@/features/profile/data/repositories/tauri-profile.repository";
import { TauriUpdaterRepository } from "@/features/updater/data/repositories/tauri-updater.repository";
import { TauriDayNoteRepository } from "@/features/work-hours/data/repositories/tauri-day-note.repository";
import { TauriTaskPresetRepository } from "@/features/work-hours/data/repositories/tauri-task-preset.repository";
import { TauriWorkEntryRepository } from "@/features/work-hours/data/repositories/tauri-work-entry.repository";

/**
 * Composition root of the frontend: the ONE place where concrete (Tauri IPC)
 * repositories are instantiated. Features only ever see the ports, through
 * `useContainer()`. A port missing here is a compile error (the `Container`
 * interface is assembled from every feature's `di.ts`).
 */
export function createTauriContainer(): Container {
  return {
    authRepository: new TauriAuthRepository(),
    profileRepository: new TauriProfileRepository(),
    profileSettingsRepository: new TauriProfileSettingsRepository(),
    presenceRepository: new TauriPresenceRepository(),
    commuteRepository: new TauriCommuteRepository(),
    emissionFactorRepository: new TauriEmissionFactorRepository(),
    taskPresetRepository: new TauriTaskPresetRepository(),
    workEntryRepository: new TauriWorkEntryRepository(),
    dayNoteRepository: new TauriDayNoteRepository(),
    exportRepository: new TauriExportRepository(),
    transferRepository: new TauriTransferRepository(),
    licenceRepository: new TauriLicenceRepository(),
    co2ReferentialRepository: new TauriCo2ReferentialRepository(),
    updaterRepository: new TauriUpdaterRepository(),
  };
}

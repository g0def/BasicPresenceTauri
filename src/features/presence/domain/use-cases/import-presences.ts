import type {
  ImportPresencesInput,
  ImportSummary,
} from "@/features/presence/domain/entities/presence";
import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

export type ImportPresencesUseCase = (
  input: ImportPresencesInput,
) => Promise<ImportSummary>;

export function makeImportPresencesUseCase(
  repo: PresenceRepository,
): ImportPresencesUseCase {
  return (input) => repo.importMany(input);
}

// Ports this feature takes from the DI container (see core/di).
import type { TransferRepository } from "@/features/profile-transfer/domain/repositories/transfer-repository";

declare module "@/core/di" {
  interface Container {
    transferRepository: TransferRepository;
  }
}

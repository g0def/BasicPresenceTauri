// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type { Co2ReferentialDto } from "@/features/methodology/data/dto/co2-referential.dto";

declare module "@/core/ipc" {
  interface CommandRegistry {
    list_co2_referential: { args: void; result: Co2ReferentialDto };
  }
}

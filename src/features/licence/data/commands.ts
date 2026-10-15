// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.

declare module "@/core/ipc" {
  interface CommandRegistry {
    /** Backend-rendered, sanitized HTML of the app licence. */
    get_licence: { args: void; result: string };
  }
}

export {};

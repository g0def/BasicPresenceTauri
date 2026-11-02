// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.

declare module "@/core/ipc" {
  interface CommandRegistry {
    /** True when a system package manager (pacman) owns this installation. */
    is_package_managed: { args: void; result: boolean };
  }
}

export {};

/** UI tick for the session countdown. */
export const SESSION_TICK_MS = 1_000;

/** Below this remaining time, the countdown switches to its warning style. */
export const COUNTDOWN_WARNING_MS = 60_000;

/** Backend command names (snake_case, matching the Rust `#[tauri::command]`s). */
export const COMMANDS = {
  accountExists: "account_exists",
  register: "register",
  login: "login",
  checkSession: "check_session",
  logout: "logout",
  createProfile: "create_profile",
  listProfiles: "list_profiles",
  updateProfile: "update_profile",
  deleteProfile: "delete_profile",
  setActiveProfile: "set_active_profile",
  setPresence: "set_presence",
  listPresences: "list_presences",
  deletePresence: "delete_presence",
} as const;

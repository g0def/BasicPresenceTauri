// Tauri isolation application (https://v2.tauri.app/concept/inter-process-communication/isolation/).
//
// Runs in a sandboxed iframe that every IPC message from the WebView goes
// through before reaching the Rust core. Deliberately dependency-free: it is
// part of the trusted computing base and must not pull any npm package.
//
// Policy:
// - app commands (`#[tauri::command]`s) must be in the allowlist below — the
//   capability ACL does not cover them, so this is their only frontend-side
//   gate. The list is checked against `lib.rs` by a Rust unit test
//   (`isolation_allowlist_matches_registered_commands`).
// - plugin commands (`plugin:<name>|<command>`) are already filtered by the
//   capability ACL (capabilities/default.json); they pass through.
// - anything else is rewritten to an unknown command, so the backend rejects
//   it with an error (the caller's promise rejects instead of hanging).

(function () {
  "use strict";

  var APP_COMMANDS = [
    "account_exists",
    "register",
    "login",
    "check_session",
    "logout",
    "change_password",
    "create_profile",
    "list_profiles",
    "update_profile",
    "delete_profile",
    "set_active_profile",
    "set_presence",
    "list_presences",
    "delete_presence",
    "list_emission_factors",
    "list_co2_referential",
    "create_commute",
    "list_commutes",
    "update_commute",
    "delete_commute",
    "get_presence_trips",
    "get_profile_settings",
    "set_profile_settings",
    "create_task_preset",
    "list_task_presets",
    "update_task_preset",
    "delete_task_preset",
    "get_work_entries",
    "set_work_entries",
    "get_work_schedule",
    "set_work_schedule",
    "get_day_note",
    "set_day_note",
    "get_licence",
    "export_profile_data",
    "export_profile_bundle",
    "inspect_profile_bundle",
    "import_profile_bundle",
  ];

  var allowed = Object.create(null);
  for (var i = 0; i < APP_COMMANDS.length; i++) allowed[APP_COMMANDS[i]] = true;

  var BLOCKED = "__blocked_by_isolation__";

  function isAllowed(cmd) {
    if (typeof cmd !== "string") return false;
    if (cmd.indexOf("plugin:") === 0) return true; // ACL-filtered by the core
    return allowed[cmd] === true;
  }

  window.__TAURI_ISOLATION_HOOK__ = function (payload) {
    if (!payload || !isAllowed(payload.cmd)) {
      payload.cmd = BLOCKED;
    }
    return payload;
  };
})();

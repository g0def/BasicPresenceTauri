/** What each calendar day cell renders: the CO2 footprint or the work hours.
 * Framework-agnostic so a domain layer can reference it without importing React;
 * the React context in `cell-display-context.ts` re-exports it. */
export type CellDisplayMode = "co2" | "hours";

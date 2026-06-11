/**
 * Suggested task colors: mid-chroma tones that stay legible on both the light
 * (#F7FFF7) and dark (#131613) backgrounds, several reused from the app's
 * semantic/chart scale in index.css. A free `<input type="color">` complements
 * the swatches, so this list is a starting point, not a constraint.
 */
export const TASK_COLOR_PALETTE = [
  "#4B7F52",
  "#7FA94E",
  "#CBA53C",
  "#CC7F3D",
  "#B24A3C",
  "#1A6F8E",
  "#5FA0B8",
  "#005377",
  "#8E6BA8",
  "#6B7B8C",
] as const;

export const DEFAULT_TASK_COLOR = TASK_COLOR_PALETTE[5];

/** A day's free-form note: the raw Markdown source (fed to the WYSIWYG editor)
 * plus the backend-rendered, sanitized HTML used for read-only display. */
export interface DayNote {
  markdown: string;
  html: string;
}

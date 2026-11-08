/** Wire shape of `DayNoteDto` (serde camelCase). `markdown` is the raw source;
 * `html` is rendered and sanitized backend-side. */
export interface DayNoteDto {
  markdown: string;
  html: string;
}

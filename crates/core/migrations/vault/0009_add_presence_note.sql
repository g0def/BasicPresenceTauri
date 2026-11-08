-- Free-form Markdown note for a day (in the ENCRYPTED vault). NULL = no note.
-- The source of truth is the raw Markdown; the HTML is rendered (and sanitized)
-- backend-side on every read, never stored.
ALTER TABLE presence ADD COLUMN note TEXT;

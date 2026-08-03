import { describe, expect, it } from "vitest";

import en from "@/core/i18n/locales/en/translation.json";
import fr from "@/core/i18n/locales/fr/translation.json";

/**
 * Guards the project rule "une clé ajoutée en FR doit l'être en EN"
 * (documentation/conformite/README.md §4, code-review-checklist.md §8).
 *
 * Nothing else can catch a FR-only key:
 *  - `src/@types/i18next.d.ts` types the key literals off the **FR** file alone,
 *    so TypeScript is blind to an EN gap;
 *  - `fallbackLng: "fr"` makes i18next serve the French string at runtime, so a
 *    component test asserting "some non-empty text" passes too.
 *
 * The catalogues are imported directly rather than through the initialised i18n
 * singleton, so this test compares the source of truth and pulls in no side
 * effects.
 */
type Tree = { [key: string]: string | Tree };

function leaves(tree: Tree, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [key, value] of Object.entries(tree)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof value === "string") out[path] = value;
    else Object.assign(out, leaves(value, path));
  }
  return out;
}

const frLeaves = leaves(fr as Tree);
const enLeaves = leaves(en as Tree);

describe("translation catalogues", () => {
  it("has no key present in only one language", () => {
    const frKeys = Object.keys(frLeaves).sort();
    const enKeys = Object.keys(enLeaves).sort();

    // Asserted as sorted arrays rather than as two `not.toContain` loops so the
    // failure message names the offending keys directly.
    expect(frKeys.filter((k) => !(k in enLeaves))).toEqual([]);
    expect(enKeys.filter((k) => !(k in frLeaves))).toEqual([]);
    expect(frKeys).toEqual(enKeys);
  });

  it("has no blank translation", () => {
    // A blank value is worse than a missing one: it produces an empty header or
    // an empty button with no fallback and no error.
    const blank = (catalogue: Record<string, string>) =>
      Object.entries(catalogue)
        .filter(([, value]) => value.trim() === "")
        .map(([key]) => key);

    expect(blank(frLeaves)).toEqual([]);
    expect(blank(enLeaves)).toEqual([]);
  });
});

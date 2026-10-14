import { describe, expect, it } from "vitest";

import i18n from "@/core/i18n";
import { MODE_LABEL_KEYS } from "@/features/commute/presentation/mode-labels";
import { buildExportLabels } from "@/features/export/presentation/build-export-labels";

// `getFixedT` rather than `changeLanguage`: the global setup pins the active
// language to FR, and switching it here would leak into every other test file.
const fr = () => buildExportLabels(i18n.getFixedT("fr"), "fr");
const en = () => buildExportLabels(i18n.getFixedT("en"), "en");

describe("buildExportLabels", () => {
  it.each([
    ["fr", fr],
    ["en", en],
  ])("resolves every label in %s", (_lang, build) => {
    const { modeNames, ...strings } = build();

    for (const [field, value] of Object.entries(strings)) {
      expect(typeof value, field).toBe("string");
      expect(value.length, field).toBeGreaterThan(0);
      // i18next echoes the key back when it is missing, so an unresolved key
      // shows up as the key itself rather than as an empty string.
      //
      // NOTE: this only proves the key is WIRED, not that it is TRANSLATED. In
      // the `en` pass a key missing from the English catalogue resolves through
      // `fallbackLng: "fr"` and returns the French string, which passes every
      // assertion here. FR/EN parity is guarded by
      // src/core/i18n/translation-parity.test.ts instead.
      expect(value, field).not.toContain("export.sheet");
      expect(value, field).not.toContain("presence.types");
    }
    expect(Object.keys(modeNames).length).toBeGreaterThan(0);
  });

  it("uses the column headers reserved by lot 3 §5.6", () => {
    expect(fr().distanceOneWay).toBe("Distance aller (km)");
    expect(en().distanceOneWay).toBe("One-way distance (km)");
    expect(fr().distanceCounted).toBe("Distance comptabilisée (km)");
    expect(en().distanceCounted).toBe("Counted distance (km)");
    expect(fr().tripSummary).toBe("Déplacement");
    expect(en().tripSummary).toBe("Commute");
    expect(fr().modeLabel).toBe("Mode (libellé)");
    expect(en().modeLabel).toBe("Mode (label)");
  });

  it("keeps the recap formatting tokens exact", () => {
    // The spaces around the joiner are load-bearing: the backend concatenates
    // legs with it verbatim.
    expect(fr().tripJoin).toBe(" + ");
    expect(en().tripJoin).toBe(" + ");
    expect(fr().unitKm).toBe("km");
    expect(en().unitKm).toBe("km");
    expect(fr().roundTripSuffix).toBe("(A/R)");
    expect(en().roundTripSuffix).toBe("(round trip)");
    expect(fr().oneWaySuffix).toBe("(aller simple)");
    expect(en().oneWaySuffix).toBe("(one way)");
  });

  it("derives the decimal separator from the language", () => {
    expect(fr().decimalSeparator).toBe(",");
    expect(en().decimalSeparator).toBe(".");
    // Unsupported tag falls back to the FR locale, like `fallbackLng`. Probed
    // with "ja" (natively ".") rather than "de" (natively ",", which would pass
    // even if the fallback were removed).
    expect(buildExportLabels(i18n.getFixedT("fr"), "ja").decimalSeparator).toBe(
      ",",
    );
  });

  it("maps every known transport mode into modeNames", () => {
    const { modeNames } = fr();

    expect(Object.keys(modeNames).sort()).toEqual(
      Object.keys(MODE_LABEL_KEYS).sort(),
    );
    // Pinned so growing the referential is a conscious decision, not a silent gap.
    expect(Object.keys(modeNames)).toHaveLength(26);
    expect(modeNames.train_sncb).toBe("Train Intercity (SNCB)");
    expect(modeNames.bike).toBe("Vélo musculaire");

    for (const [modeId, name] of Object.entries(modeNames)) {
      expect(name, modeId).not.toContain("commute.modes.");
    }
  });

  it("resolves modeNames for the requested language, not the active one", () => {
    // The global setup pins the active language to FR; this must still be EN.
    expect(en().modeNames.bike).toBe("Bicycle");
  });
});

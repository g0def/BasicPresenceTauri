import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import { defaultNS, resources, supportedLngs } from "@/core/i18n/resources";

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "fr",
    supportedLngs: [...supportedLngs],
    // Map region-specific OS locales (e.g. "en-US", "fr-CA") down to a
    // supported base language so first-launch detection resolves correctly.
    load: "languageOnly",
    defaultNS,
    interpolation: {
      // React already escapes values, so i18next must not double-escape.
      escapeValue: false,
    },
    detection: {
      // localStorage holds the user's explicit choice; navigator falls back to
      // the OS locale exposed by the Tauri webview on first launch.
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
    },
    react: {
      // Resources are bundled, so there is nothing to wait for.
      useSuspense: false,
    },
  });

// Keep <html lang> in sync for accessibility and styling hooks.
document.documentElement.lang = i18n.resolvedLanguage ?? "fr";
i18n.on("languageChanged", (lng) => {
  document.documentElement.lang = lng;
});

export default i18n;

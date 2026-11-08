import fr from "@/core/i18n/locales/fr/translation.json";
import en from "@/core/i18n/locales/en/translation.json";

export const defaultNS = "translation" as const;

export const resources = {
  fr: { translation: fr },
  en: { translation: en },
} as const;

export const supportedLngs = ["fr", "en"] as const;

export type SupportedLanguage = (typeof supportedLngs)[number];
